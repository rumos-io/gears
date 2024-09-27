// #![cfg(feature = "it")]

use std::str::FromStr;

use bank::{
    cli::query::{BalancesCommand, BankCommands as BankQueryCommands, BankQueryCli},
    types::query::QueryAllBalancesResponse,
};
use gaia_rs::{client::GaiaQueryCommands, query::GaiaQueryResponse};
use gears::types::{address::AccAddress, base::coin::UnsignedCoin, denom::Denom};
use utilities::GaiaNode;

#[path = "./utilities.rs"]
mod utilities;

#[test]
fn balances_query() -> anyhow::Result<()> {
    let gaia = GaiaNode::run()?;

    let cmd = GaiaQueryCommands::Bank(BankQueryCli {
        command: BankQueryCommands::Balances(BalancesCommand {
            address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")?,
            pagination: None,
        }),
    });

    let result = gaia.query(cmd)?;

    let expected = GaiaQueryResponse::Bank(bank::cli::query::BankQueryResponse::Balances(
        QueryAllBalancesResponse {
            balances: vec![UnsignedCoin {
                denom: Denom::from_str("uatom")?,
                amount: 990_000_000_000_u64.into(),
            }],
            pagination: None,
        },
    ));

    assert_eq!(result, expected);

    Ok(())
}
