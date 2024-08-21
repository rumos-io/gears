use crate::collect_txs::CollectGentxCmd;

#[derive(Debug, Clone)]
pub enum GenesisCmd {
    /// Collect genesis txs and output a genesis.json file
    CollectGentxs(CollectGentxCmd),
}
