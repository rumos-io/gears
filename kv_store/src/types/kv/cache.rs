use std::collections::{BTreeMap, HashSet};

use database::Database;

use crate::{types::prefix::immutable::ImmutablePrefixStore, TransactionStore};

use super::{immutable::KVStore, KVBank};

impl<DB: Database> KVBank<DB, TransactionStore> {
    pub fn commit(&mut self) -> (BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>) {
        let (mut block_set, mut block_del) = self.block.take();
        let (tx_set, tx_del) = self.tx.take();

        block_set.extend(tx_set);
        block_del.extend(tx_del);

        (block_set, block_del)
    }

    pub fn prefix_store<I: IntoIterator<Item = u8>>(
        &self,
        prefix: I,
    ) -> ImmutablePrefixStore<'_, DB> {
        ImmutablePrefixStore {
            store: KVStore::from(self),
            prefix: prefix.into_iter().collect(),
        }
    }
}
