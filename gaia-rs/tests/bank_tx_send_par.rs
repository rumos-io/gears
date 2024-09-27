// #![cfg(feature = "it")]

use std::str::FromStr;

use bank::cli::tx::{BankCommands, BankTxCli};
use gaia_rs::client::GaiaTxCommands;
use gears::{
    tendermint::rpc::response::tx::broadcast::Response,
    types::{address::AccAddress, base::coin::UnsignedCoin},
};
use utilities::GaiaNode;

#[path = "./utilities.rs"]
mod utilities;

/// NOTE: This test doesn't check resulted hash and what events occurred. It tries to check that our app works under *sign* some load
#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn send_tx_in_parallel() -> anyhow::Result<()> {
    let gaia = GaiaNode::run()?;

    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    GaiaNode::accounts()
        .into_par_iter()
        .try_for_each(|(key_name, _)| {
            let cmd = GaiaTxCommands::Bank(BankTxCli {
                command: BankCommands::Send {
                    to_address: AccAddress::from_bech32(
                        "cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut",
                    )?,
                    amount: UnsignedCoin::from_str("10uatom")?,
                },
            });

            let responses = gaia
                .tx(cmd, key_name)?
                .broadcast()
                .expect("broadcast tx inside");

            assert_eq!(responses.len(), 1);
            let Response {
                check_tx,
                deliver_tx,
                hash: _,
                height: _,
            } = &responses[0];

            assert!(check_tx.code.is_ok());
            assert!(deliver_tx.code.is_ok());

            anyhow::Ok(())
        })?;

    Ok(())
}
