use std::str::FromStr;

use bank::{
    cli::query::{
        BalancesCommand, BankCommands as BankQueryCommands, BankQueryCli, BankQueryResponse,
    },
    types::query::{QueryAllBalancesResponse, QueryDenomsMetadataResponse},
};
use gaia_rs::{query::GaiaQueryResponse, GaiaCoreClient};
use gears::{
    commands::client::query::{run_query, QueryCommand}, config::DEFAULT_TENDERMINT_RPC_ADDRESS, ibc::address::AccAddress, proto_types::Denom, types::base::coin::Coin
};
use utilities::run_gaia_and_tendermint;

#[path = "./utilities.rs"]
mod utilities;

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn balances_query() -> anyhow::Result<()> {
    let (_tendermint, _server_thread) = run_gaia_and_tendermint()?;

    let query = BalancesCommand {
        address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")?,
    };

    let result = run_query(
        QueryCommand {
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            height: None,
            inner: gaia_rs::client::GaiaQueryCommands::Bank(BankQueryCli {
                command: BankQueryCommands::Balances(query),
            }),
        },
        &GaiaCoreClient,
    )?;

    let expected = GaiaQueryResponse::Bank(bank::cli::query::BankQueryResponse::Balances(
        QueryAllBalancesResponse {
            balances: vec![Coin {
                denom: Denom::from_str("uatom")?,
                amount: 34_u32.into(),
            }],
            pagination: None,
        },
    ));

    assert_eq!(result, expected);

    Ok(())
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn denom_query() -> anyhow::Result<()> {
    let (_tendermint, _server_thread) = run_gaia_and_tendermint()?;

    let result = run_query(
        QueryCommand {
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            height: None,
            inner: gaia_rs::client::GaiaQueryCommands::Bank(BankQueryCli {
                command: BankQueryCommands::DenomMetadata,
            }),
        },
        &GaiaCoreClient,
    )?;

    let expected = GaiaQueryResponse::Bank(BankQueryResponse::DenomMetadata(
        QueryDenomsMetadataResponse {
            metadatas: Vec::new(),
            pagination: None,
        },
    ));

    assert_eq!(result, expected);

    Ok(())
}

// TODO: Need to rework keys so cli wouldn't be required
// #[test]
// fn send_tx() -> anyhow::Result<()> {
//     let ( tendermint, _server_thread ) = run_gaia_and_tendermint()?;

//     let tx_cmd = BankCommands::Send {
//         to_address: AccAddress::from_bech32("cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut")?,
//         amount: Coin::from_str("10uatom")?,
//     };

//     let _result = run_tx(
//         TxCommand {
//             home : tendermint.1.to_path_buf(),
//             node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
//             from_key: "alice".to_owned(),
//             chain_id: Id::try_from("test-chain")?,
//             fee: None,
//             keyring_backend: KeyringBackend::File,
//             inner: GaiaTxCommands::Bank(BankTxCli { command: tx_cmd }),
//         },
//         &GaiaCore,
//     )?;

//     Ok(())
// }
