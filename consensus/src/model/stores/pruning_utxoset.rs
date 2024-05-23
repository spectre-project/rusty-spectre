use std::sync::Arc;

use rocksdb::WriteBatch;
use spectre_database::prelude::CachePolicy;
use spectre_database::prelude::StoreResult;
use spectre_database::prelude::DB;
use spectre_database::prelude::{BatchDbWriter, CachedDbItem};
use spectre_database::registry::DatabaseStorePrefixes;
use spectre_hashes::Hash;

use super::utxo_set::DbUtxoSetStore;

/// Used in order to group stores related to the pruning point utxoset under a single lock
pub struct PruningUtxosetStores {
    pub utxo_set: DbUtxoSetStore,
    utxoset_position_access: CachedDbItem<Hash>,
}

impl PruningUtxosetStores {
    pub fn new(db: Arc<DB>, utxoset_cache_policy: CachePolicy) -> Self {
        Self {
            utxo_set: DbUtxoSetStore::new(db.clone(), utxoset_cache_policy, DatabaseStorePrefixes::PruningUtxoset.into()),
            utxoset_position_access: CachedDbItem::new(db, DatabaseStorePrefixes::PruningUtxosetPosition.into()),
        }
    }

    /// Represents the exact point of the current pruning point utxoset. Used it order to safely
    /// progress the pruning point utxoset in batches and to allow recovery if the process crashes
    /// during the pruning point utxoset movement
    pub fn utxoset_position(&self) -> StoreResult<Hash> {
        self.utxoset_position_access.read()
    }

    pub fn set_utxoset_position(&mut self, batch: &mut WriteBatch, pruning_utxoset_position: Hash) -> StoreResult<()> {
        self.utxoset_position_access.write(BatchDbWriter::new(batch), &pruning_utxoset_position)
    }
}
