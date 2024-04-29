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
    core::address::AccAddress,
    tendermint::{
        abci::{Event, EventAttribute},
        rpc::response::tx::broadcast::Response,
        types::chain_id::ChainId,
    },
    types::{base::coin::Coin, denom::Denom},
};
use utilities::run_gaia_and_tendermint;

use crate::utilities::{key_add, KEY_NAME};

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
            inner: WrappedGaiaQueryCommands(GaiaQueryCommands::Bank(BankQueryCli {
                command: BankQueryCommands::Balances(query),
            })),
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
            inner: WrappedGaiaQueryCommands(GaiaQueryCommands::Bank(BankQueryCli {
                command: BankQueryCommands::DenomMetadata,
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
    let (tendermint, _server_thread) = run_gaia_and_tendermint()?;

    let tx_cmd = BankCommands::Send {
        to_address: AccAddress::from_bech32("cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut")?,
        amount: Coin::from_str("10uatom")?,
    };

    key_add(tendermint.1.to_path_buf())?;

    let Response {
        check_tx: _,
        deliver_tx,
        hash,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Bank(BankTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;

    let expected_hash = data_encoding::HEXUPPER
        .decode("ADF35F8DCA01BD6A44F1260EDADA79C9FBED1C8C9F5431C3D192A2C1D0AF209D".as_bytes())?;

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
