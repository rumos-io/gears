#![cfg(feature = "it")]

use std::str::FromStr;

use bank::{
    cli::{
        query::{
            BalancesCommand, BankCommands as BankQueryCommands, BankQueryCli, BankQueryResponse,
        },
        tx::{BankCommands, BankTxCli},
    },
    types::query::{QueryAllBalancesResponse, QueryDenomsMetadataResponse},
};
use gaia_rs::{
    client::{GaiaQueryCommands, GaiaTxCommands, WrappedGaiaQueryCommands, WrappedGaiaTxCommands},
    query::GaiaQueryResponse,
    GaiaCoreClient,
};
use gears::{
    commands::client::{
        keys::KeyringBackend,
        query::{run_query, QueryCommand},
        tx::{run_tx, Keyring, LocalInfo, TxCommand},
    },
    config::DEFAULT_TENDERMINT_RPC_ADDRESS,
    tendermint::{
        abci::{Event, EventAttribute},
        rpc::response::tx::broadcast::Response,
        types::chain_id::ChainId,
    },
    types::{address::AccAddress, base::coin::UnsignedCoin, denom::Denom},
};
use utilities::tendermint;

use crate::utilities::KEY_NAME;

#[path = "./utilities.rs"]
mod utilities;

#[test]
fn balances_query() -> anyhow::Result<()> {
    let _tendermint = tendermint();

    let query = BalancesCommand {
        address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")?,
        pagination: None,
    };

    let result = run_query(
        QueryCommand {
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            height: None,
            inner: WrappedGaiaQueryCommands(GaiaQueryCommands::Bank(BankQueryCli {
                command: BankQueryCommands::Balances(query),
            })),
        },
        &GaiaCoreClient,
    )?;

    let expected = GaiaQueryResponse::Bank(bank::cli::query::BankQueryResponse::Balances(
        QueryAllBalancesResponse {
            balances: vec![UnsignedCoin {
                denom: Denom::from_str("uatom")?,
                amount: 200_000_000_u32.into(),
            }],
            pagination: None,
        },
    ));

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn denom_query() -> anyhow::Result<()> {
    let _tendermint = tendermint();

    let result = run_query(
        QueryCommand {
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            height: None,
            inner: WrappedGaiaQueryCommands(GaiaQueryCommands::Bank(BankQueryCli {
                command: BankQueryCommands::DenomMetadata { pagination: None },
            })),
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

#[test]
fn send_tx() -> anyhow::Result<()> {
    let tendermint = tendermint();

    let tx_cmd = BankCommands::Send {
        to_address: AccAddress::from_bech32("cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut")?,
        amount: UnsignedCoin::from_str("10uatom")?,
    };

    let responses = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),

            }),
            home: tendermint.1.to_path_buf(),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fees: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Bank(BankTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?
    .broadcast()
    .expect("broadcast tx inside");

    assert_eq!(responses.len(), 1);
    let Response {
        check_tx: _,
        deliver_tx,
        hash,
        height: _,
    } = &responses[0];

    let expected_hash = data_encoding::HEXUPPER
        .decode("13BB2C6817D0EDA960EDB0C6D6D5CB752D341BB603EF4BCE990F4EA5A99500C1".as_bytes())?;

    assert_eq!(&expected_hash, hash.as_bytes());
    assert!(deliver_tx.code.is_ok());

    let expected_events = [Event {
        kind: "transfer".to_owned(),
        attributes: vec![
            EventAttribute {
                key: "recipient".to_owned(),
                value: "cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut".to_owned(),
                index: true,
            },
            EventAttribute {
                key: "sender".to_owned(),
                value: "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_owned(),
                index: true,
            },
            EventAttribute {
                key: "amount".to_owned(),
                value: "10".to_owned(),
                index: true,
            },
        ],
    }];

    assert_eq!(expected_events.as_slice(), deliver_tx.events.as_slice());

    Ok(())
}

// /// NOTE: This test doesn't check resulted hash and what events occured. It tries to check that our app works under *sign* some load
// #[test]
// #[ignore = "rust usually run test in || while this tests be started ony by one"]
// fn send_tx_in_parallel() -> anyhow::Result<()> {
//     let coin = default_coin(200_000_000_u32);

//     let addresses = [
//         (random_address(), coin.clone()),
//         (random_address(), coin.clone()),
//         (random_address(), coin.clone()),
//         (random_address(), coin.clone()),
//         (random_address(), coin.clone()),
//         (random_address(), coin.clone()),
//         (random_address(), coin.clone()),
//         (random_address(), coin.clone()),
//         (random_address(), coin.clone()),
//         (random_address(), coin),
//     ];

//     let (tendermint, _server_thread) = run_gaia_and_tendermint(addresses.clone())?;

//     use rayon::iter::{IntoParallelIterator, ParallelIterator};

//     addresses.into_par_iter().try_for_each(|(_, _)| {
//         let tx_cmd = BankCommands::Send {
//             to_address: AccAddress::from_bech32("cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut")?,
//             amount: UnsignedCoin::from_str("10uatom")?,
//         };

//         let responses = run_tx(
//             TxCommand {
//                 keyring: Keyring::Local(LocalInfo {
//                     keyring_backend: KeyringBackend::Test,
//                     from_key: KEY_NAME.to_owned(),
//                     home: tendermint.1.to_path_buf(),
//                 }),
//                 node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
//                 chain_id: ChainId::from_str("test-chain")?,
//                 fees: None,
//                 inner: WrappedGaiaTxCommands(GaiaTxCommands::Bank(BankTxCli { command: tx_cmd })),
//             },
//             &GaiaCoreClient,
//         )?;
//         assert_eq!(responses.len(), 1);
//         let Response {
//             check_tx,
//             deliver_tx,
//             hash: _,
//             height: _,
//         } = &responses[0];

//         assert!(check_tx.code.is_ok());
//         assert!(deliver_tx.code.is_ok());

//         anyhow::Ok(())
//     })?;

//     Ok(())
// }
