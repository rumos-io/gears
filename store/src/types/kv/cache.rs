use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct KVStoreCache {
    pub(crate) block: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) tx: BTreeMap<Vec<u8>, Vec<u8>>,

    pub(crate) delete: HashSet<Vec<u8>>,
}

impl KVStoreCache {
    pub(crate) fn tx_upgrade_to_block(&mut self) {
        let tx_map = std::mem::take(&mut self.tx);

        for (key, value) in tx_map {
            let _ = self.block.insert(key, value);
        }
    }

    // TODO:NOW Awful name
    pub(crate) fn commit(&mut self) -> (BTreeMap<Vec<u8>, Vec<u8>>, HashSet<Vec<u8>>) {
        let tx_map = std::mem::take(&mut self.tx);
        let mut block_map = std::mem::take(&mut self.block);

        block_map.extend(tx_map);
        (block_map, std::mem::take(&mut self.delete))
    }
}
