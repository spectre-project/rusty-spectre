use std::sync::Arc;

use spectre_consensus_core::{
    blockhash::{self, BlockHashExtensions, BlockHashes},
    BlockHashMap, BlockLevel, BlueWorkType, HashMapCustomHasher,
};
use spectre_hashes::Hash;
use spectre_utils::refs::Refs;

use crate::{
    model::{
        services::reachability::ReachabilityService,
        stores::{
            ghostdag::{GhostdagData, GhostdagStoreReader, HashKTypeMap, KType},
            headers::HeaderStoreReader,
            relations::RelationsStoreReader,
        },
    },
    processes::difficulty::{calc_work, level_work},
};

use super::ordering::*;

#[derive(Clone)]
pub struct GhostdagManager<T: GhostdagStoreReader, S: RelationsStoreReader, U: ReachabilityService, V: HeaderStoreReader> {
    genesis_hash: Hash,
    pub(super) k: KType,
    pub(super) ghostdag_store: Arc<T>,
    pub(super) relations_store: S,
    pub(super) headers_store: Arc<V>,
    pub(super) reachability_service: U,

    /// Level work is a lower-bound for the amount of work represented by each block.
    /// When running GD for higher-level sub-DAGs, this value should be set accordingly
    /// to the work represented by that level, and then used as a lower bound
    /// for the work calculated from header bits (which depends on current difficulty).
    /// For instance, assuming level 80 (i.e., pow hash has at least 80 zeros) is always
    /// above the difficulty target, all blocks in it should represent the same amount of
    /// work regardless of whether current difficulty requires 20 zeros or 25 zeros.  
    level_work: BlueWorkType,
}

impl<T: GhostdagStoreReader, S: RelationsStoreReader, U: ReachabilityService, V: HeaderStoreReader> GhostdagManager<T, S, U, V> {
    pub fn new(
        genesis_hash: Hash,
        k: KType,
        ghostdag_store: Arc<T>,
        relations_store: S,
        headers_store: Arc<V>,
        reachability_service: U,
    ) -> Self {
        // For ordinary GD, always keep level_work=0 so the lower bound is ineffective
        Self { genesis_hash, k, ghostdag_store, relations_store, reachability_service, headers_store, level_work: 0.into() }
    }

    pub fn with_level(
        genesis_hash: Hash,
        k: KType,
        ghostdag_store: Arc<T>,
        relations_store: S,
        headers_store: Arc<V>,
        reachability_service: U,
        level: BlockLevel,
        max_block_level: BlockLevel,
    ) -> Self {
        Self {
            genesis_hash,
            k,
            ghostdag_store,
            relations_store,
            reachability_service,
            headers_store,
            level_work: level_work(level, max_block_level),
        }
    }

    pub fn genesis_ghostdag_data(&self) -> GhostdagData {
        GhostdagData::new(
            0,
            Default::default(),
            blockhash::ORIGIN,
            BlockHashes::new(Vec::new()),
            BlockHashes::new(Vec::new()),
            HashKTypeMap::new(BlockHashMap::new()),
        )
    }

    pub fn origin_ghostdag_data(&self) -> Arc<GhostdagData> {
        Arc::new(GhostdagData::new(
            0,
            Default::default(),
            0.into(),
            BlockHashes::new(Vec::new()),
            BlockHashes::new(Vec::new()),
            HashKTypeMap::new(BlockHashMap::new()),
        ))
    }

    pub fn find_selected_parent(&self, parents: impl IntoIterator<Item = Hash>) -> Hash {
        parents
            .into_iter()
            .map(|parent| SortableBlock { hash: parent, blue_work: self.ghostdag_store.get_blue_work(parent).unwrap() })
            .max()
            .unwrap()
            .hash
    }

    /// Runs the GHOSTDAG protocol and calculates the block GhostdagData by the given parents.
    /// The function calculates mergeset blues by iterating over the blocks in
    /// the anticone of the new block selected parent (which is the parent with the
    /// highest blue work) and adds any block to the blue set if by adding
    /// it these conditions will not be violated:
    ///
    /// 1) |anticone-of-candidate-block ∩ blue-set-of-new-block| ≤ K
    ///
    /// 2) For every blue block in blue-set-of-new-block:
    ///    |(anticone-of-blue-block ∩ blue-set-new-block) ∪ {candidate-block}| ≤ K.
    ///    We validate this condition by maintaining a map blues_anticone_sizes for
    ///    each block which holds all the blue anticone sizes that were affected by
    ///    the new added blue blocks.
    ///    So to find out what is |anticone-of-blue ∩ blue-set-of-new-block| we just iterate in
    ///    the selected parent chain of the new block until we find an existing entry in
    ///    blues_anticone_sizes.
    ///
    /// For further details see the article <https://eprint.iacr.org/2018/104.pdf>
    pub fn ghostdag(&self, parents: &[Hash]) -> GhostdagData {
        assert!(!parents.is_empty(), "genesis must be added via a call to init");

        // Run the GHOSTDAG parent selection algorithm
        let selected_parent = self.find_selected_parent(parents.iter().copied());
        // Initialize new GHOSTDAG block data with the selected parent
        let mut new_block_data = GhostdagData::new_with_selected_parent(selected_parent, self.k);
        // Get the mergeset in consensus-agreed topological order (topological here means forward in time from blocks to children)
        let ordered_mergeset = self.ordered_mergeset_without_selected_parent(selected_parent, parents);

        for blue_candidate in ordered_mergeset.iter().cloned() {
            let coloring = self.check_blue_candidate(&new_block_data, blue_candidate);

            if let ColoringOutput::Blue(blue_anticone_size, blues_anticone_sizes) = coloring {
                // No k-cluster violation found, we can now set the candidate block as blue
                new_block_data.add_blue(blue_candidate, blue_anticone_size, &blues_anticone_sizes);
            } else {
                new_block_data.add_red(blue_candidate);
            }
        }

        // Handle the special case of origin children first
        if selected_parent.is_origin() {
            // ORIGIN is always a single parent so both blue score and work should remain zero
            return new_block_data;
        }

        let blue_score = self.ghostdag_store.get_blue_score(selected_parent).unwrap() + new_block_data.mergeset_blues.len() as u64;

        let added_blue_work: BlueWorkType = new_block_data
            .mergeset_blues
            .iter()
            .cloned()
            .map(|hash| calc_work(self.headers_store.get_bits(hash).unwrap()).max(self.level_work))
            .sum();
        let blue_work: BlueWorkType = self.ghostdag_store.get_blue_work(selected_parent).unwrap() + added_blue_work;

        new_block_data.finalize_score_and_work(blue_score, blue_work);

        new_block_data
    }

    fn check_blue_candidate_with_chain_block(
        &self,
        new_block_data: &GhostdagData,
        chain_block: &ChainBlock,
        blue_candidate: Hash,
        candidate_blues_anticone_sizes: &mut BlockHashMap<KType>,
        candidate_blue_anticone_size: &mut KType,
    ) -> ColoringState {
        // If blue_candidate is in the future of chain_block, it means
        // that all remaining blues are in the past of chain_block and thus
        // in the past of blue_candidate. In this case we know for sure that
        // the anticone of blue_candidate will not exceed K, and we can mark
        // it as blue.
        //
        // The new block is always in the future of blue_candidate, so there's
        // no point in checking it.

        // We check if chain_block is not the new block by checking if it has a hash.
        if let Some(hash) = chain_block.hash {
            if self.reachability_service.is_dag_ancestor_of(hash, blue_candidate) {
                return ColoringState::Blue;
            }
        }

        for &block in chain_block.data.mergeset_blues.iter() {
            // Skip blocks that exist in the past of blue_candidate.
            if self.reachability_service.is_dag_ancestor_of(block, blue_candidate) {
                continue;
            }

            candidate_blues_anticone_sizes.insert(block, self.blue_anticone_size(block, new_block_data));

            *candidate_blue_anticone_size += 1;
            if *candidate_blue_anticone_size > self.k {
                // k-cluster violation: The candidate's blue anticone exceeded k
                return ColoringState::Red;
            }

            if *candidate_blues_anticone_sizes.get(&block).unwrap() == self.k {
                // k-cluster violation: A block in candidate's blue anticone already
                // has k blue blocks in its own anticone
                return ColoringState::Red;
            }

            // This is a sanity check that validates that a blue
            // block's blue anticone is not already larger than K.
            assert!(*candidate_blues_anticone_sizes.get(&block).unwrap() <= self.k, "found blue anticone larger than K");
        }

        ColoringState::Pending
    }

    /// Returns the blue anticone size of `block` from the worldview of `context`.
    /// Expects `block` to be in the blue set of `context`
    fn blue_anticone_size(&self, block: Hash, context: &GhostdagData) -> KType {
        let mut current_blues_anticone_sizes = HashKTypeMap::clone(&context.blues_anticone_sizes);
        let mut current_selected_parent = context.selected_parent;
        loop {
            if let Some(size) = current_blues_anticone_sizes.get(&block) {
                return *size;
            }

            if current_selected_parent == self.genesis_hash || current_selected_parent == blockhash::ORIGIN {
                panic!("block {block} is not in blue set of the given context");
            }

            current_blues_anticone_sizes = self.ghostdag_store.get_blues_anticone_sizes(current_selected_parent).unwrap();
            current_selected_parent = self.ghostdag_store.get_selected_parent(current_selected_parent).unwrap();
        }
    }

    fn check_blue_candidate(&self, new_block_data: &GhostdagData, blue_candidate: Hash) -> ColoringOutput {
        // The maximum length of new_block_data.mergeset_blues can be K+1 because
        // it contains the selected parent.
        if new_block_data.mergeset_blues.len() as KType == self.k + 1 {
            return ColoringOutput::Red;
        }

        let mut candidate_blues_anticone_sizes: BlockHashMap<KType> = BlockHashMap::with_capacity(self.k as usize);
        // Iterate over all blocks in the blue past of the new block that are not in the past
        // of blue_candidate, and check for each one of them if blue_candidate potentially
        // enlarges their blue anticone to be over K, or that they enlarge the blue anticone
        // of blue_candidate to be over K.
        let mut chain_block = ChainBlock { hash: None, data: new_block_data.into() };
        let mut candidate_blue_anticone_size: KType = 0;

        loop {
            let state = self.check_blue_candidate_with_chain_block(
                new_block_data,
                &chain_block,
                blue_candidate,
                &mut candidate_blues_anticone_sizes,
                &mut candidate_blue_anticone_size,
            );

            match state {
                ColoringState::Blue => return ColoringOutput::Blue(candidate_blue_anticone_size, candidate_blues_anticone_sizes),
                ColoringState::Red => return ColoringOutput::Red,
                ColoringState::Pending => (), // continue looping
            }

            chain_block = ChainBlock {
                hash: Some(chain_block.data.selected_parent),
                data: self.ghostdag_store.get_data(chain_block.data.selected_parent).unwrap().into(),
            }
        }
    }
}

/// Chain block with attached ghostdag data
struct ChainBlock<'a> {
    hash: Option<Hash>, // if set to `None`, signals being the new block
    data: Refs<'a, GhostdagData>,
}

/// Represents the intermediate GHOSTDAG coloring state for the current candidate
enum ColoringState {
    Blue,
    Red,
    Pending,
}

/// Represents the final output of GHOSTDAG coloring for the current candidate
enum ColoringOutput {
    Blue(KType, BlockHashMap<KType>), // (blue anticone size, map of blue anticone sizes for each affected blue)
    Red,
}
