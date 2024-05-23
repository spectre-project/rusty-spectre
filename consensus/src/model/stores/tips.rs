use std::sync::Arc;

use rocksdb::WriteBatch;
use spectre_consensus_core::BlockHashSet;
use spectre_consensus_core::BlockHasher;
use spectre_database::prelude::CachedDbSetItem;
use spectre_database::prelude::DbWriter;
use spectre_database::prelude::ReadLock;
use spectre_database::prelude::StoreResult;
use spectre_database::prelude::StoreResultExtensions;
use spectre_database::prelude::DB;
use spectre_database::prelude::{BatchDbWriter, DirectDbWriter};
use spectre_database::registry::DatabaseStorePrefixes;
use spectre_hashes::Hash;

/// Reader API for `TipsStore`.
pub trait TipsStoreReader {
    fn get(&self) -> StoreResult<ReadLock<BlockHashSet>>;
}

pub trait TipsStore: TipsStoreReader {
    fn add_tip(&mut self, new_tip: Hash, new_tip_parents: &[Hash]) -> StoreResult<ReadLock<BlockHashSet>>;
    fn add_tip_batch(
        &mut self,
        batch: &mut WriteBatch,
        new_tip: Hash,
        new_tip_parents: &[Hash],
    ) -> StoreResult<ReadLock<BlockHashSet>> {
        self.add_tip_with_writer(BatchDbWriter::new(batch), new_tip, new_tip_parents)
    }
    fn add_tip_with_writer(
        &mut self,
        writer: impl DbWriter,
        new_tip: Hash,
        new_tip_parents: &[Hash],
    ) -> StoreResult<ReadLock<BlockHashSet>>;
    fn prune_tips_batch(&mut self, batch: &mut WriteBatch, pruned_tips: &[Hash]) -> StoreResult<()> {
        self.prune_tips_with_writer(BatchDbWriter::new(batch), pruned_tips)
    }
    fn prune_tips_with_writer(&mut self, writer: impl DbWriter, pruned_tips: &[Hash]) -> StoreResult<()>;
}

/// A DB + cache implementation of `TipsStore` trait
#[derive(Clone)]
pub struct DbTipsStore {
    db: Arc<DB>,
    access: CachedDbSetItem<Hash, BlockHasher>,
}

impl DbTipsStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db: Arc::clone(&db), access: CachedDbSetItem::new(db, DatabaseStorePrefixes::Tips.into()) }
    }

    pub fn clone_with_new_cache(&self) -> Self {
        Self::new(Arc::clone(&self.db))
    }

    pub fn is_initialized(&self) -> bool {
        self.access.read().unwrap_option().is_some()
    }

    pub fn init_batch(&mut self, batch: &mut WriteBatch, initial_tips: &[Hash]) -> StoreResult<()> {
        self.access.update(BatchDbWriter::new(batch), initial_tips, &[])?;
        Ok(())
    }
}

impl TipsStoreReader for DbTipsStore {
    fn get(&self) -> StoreResult<ReadLock<BlockHashSet>> {
        self.access.read()
    }
}

impl TipsStore for DbTipsStore {
    fn add_tip(&mut self, new_tip: Hash, new_tip_parents: &[Hash]) -> StoreResult<ReadLock<BlockHashSet>> {
        self.access.update(DirectDbWriter::new(&self.db), &[new_tip], new_tip_parents)
    }

    fn add_tip_with_writer(
        &mut self,
        writer: impl DbWriter,
        new_tip: Hash,
        new_tip_parents: &[Hash],
    ) -> StoreResult<ReadLock<BlockHashSet>> {
        // New tip parents are no longer tips and hence removed
        self.access.update(writer, &[new_tip], new_tip_parents)
    }

    fn prune_tips_with_writer(&mut self, writer: impl DbWriter, pruned_tips: &[Hash]) -> StoreResult<()> {
        if pruned_tips.is_empty() {
            return Ok(());
        }
        self.access.update(writer, &[], pruned_tips)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_database::{create_temp_db, prelude::ConnBuilder};

    #[test]
    fn test_update_tips() {
        let (_lifetime, db) = create_temp_db!(ConnBuilder::default().with_files_limit(10));
        let mut store = DbTipsStore::new(db.clone());
        store.add_tip(1.into(), &[]).unwrap();
        store.add_tip(3.into(), &[]).unwrap();
        store.add_tip(5.into(), &[]).unwrap();
        let tips = store.add_tip(7.into(), &[3.into(), 5.into()]).unwrap();
        assert_eq!(tips.read().clone(), BlockHashSet::from_iter([1.into(), 7.into()]));
    }
}
