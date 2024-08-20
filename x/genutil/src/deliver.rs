// use std::path::Path;

// use gears::{config::Config, store::StoreKey};

// use crate::{balances_iter::GenesisBalanceIter, collect_txs::collect_txs};

// pub fn deliver_tx<SK: StoreKey, AC: Default + Clone>(
//     app_config: &mut Config<AC>,
//     sk: &SK,
//     genesis_path: impl AsRef<Path>,
//     dir: impl AsRef<Path>,
//     moniker: String,
// ) -> anyhow::Result<()> {
//     let (persisted_peers, app_gen_txs) =
//         collect_txs(dir, moniker, GenesisBalanceIter::new(sk, genesis_path)?)?;

//     if app_gen_txs.is_empty() {
//         Err(anyhow::anyhow!("there must be at least one genesis tx"))?
//     }

//     Ok(())
// }
