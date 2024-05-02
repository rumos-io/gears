use std::collections::{BTreeMap, HashSet};

/// Storage for store cache
#[derive(Debug, Clone, Default)]
pub struct KVStoreCache {
    pub(crate) block: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) tx: BTreeMap<Vec<u8>, Vec<u8>>,

    pub(crate) delete: HashSet<Vec<u8>>,
}

impl KVStoreCache {
    /// Take TX cache and push it to BLOCK
    pub(crate) fn tx_upgrade_to_block(&mut self) {
        let tx_map = std::mem::take(&mut self.tx);

        self.block.extend(tx_map)
    }

    /// Take out all cache from storages. TX cache overwrites BLOCK cache
    pub(crate) fn take(&mut self) -> (BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>) {
        let tx_map = std::mem::take(&mut self.tx);
        let mut block_map = std::mem::take(&mut self.block);

        block_map.extend(tx_map);
        (block_map, std::mem::take(&mut self.delete))
    }
}
