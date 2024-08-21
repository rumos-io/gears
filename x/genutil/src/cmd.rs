use gears::commands::client::tx::TxCommand;

use crate::{collect_txs::CollectGentxCmd, gentx::GentxCmd};

#[derive(Debug, Clone)]
pub enum GenesisCmd {
    /// Collect genesis txs and output a genesis.json file
    CollectGentxs(CollectGentxCmd),
    Gentx(TxCommand<GentxCmd>),
}
