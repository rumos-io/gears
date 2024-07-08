use database::{prefix::PrefixDB, Database};

use crate::{kv::transaction::TransactionKVBank, StoreKey};

use super::*;

#[derive(Debug)]
pub struct TransactionStore<DB, SK>(pub(crate) HashMap<SK, TransactionKVBank<PrefixDB<DB>>>);

impl<SK, DB> MultiBankBackend<DB, SK> for TransactionStore<DB, SK> {
    type Bank = TransactionKVBank<PrefixDB<DB>>;

    fn stores(&self) -> &HashMap<SK, Self::Bank> {
        &self.0
    }

    fn stores_mut(&mut self) -> &mut HashMap<SK, Self::Bank> {
        &mut self.0
    }
}

impl<DB: Database, SK: StoreKey> MultiBank<DB, SK, TransactionStore<DB, SK>> {
    // pub fn commit(&mut self) -> CacheCommitList<SK> {
    //     let mut map: Vec<(SK, BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>)> =
    //         Vec::with_capacity(self.backend.0.len());

    //     for (sk, store) in &mut self.backend.0 {
    //         let (set, delete) = store.commit();
    //         map.push((sk.to_owned(), set, delete));
    //     }

    //     CacheCommitList(map)
    // }
}
