// #![cfg(feature = "it")]

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
    client::{GaiaQueryCommands, GaiaTxCommands},
    query::GaiaQueryResponse,
};
use gears::{
    tendermint::{
        abci::{Event, EventAttribute},
        rpc::response::tx::broadcast::Response,
    },
    types::{address::AccAddress, base::coin::UnsignedCoin, denom::Denom},
};
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

#[test]
fn denom_query() -> anyhow::Result<()> {
    let gaia = GaiaNode::run()?;

    let cmd = GaiaQueryCommands::Bank(BankQueryCli {
        command: BankQueryCommands::DenomMetadata { pagination: None },
    });

    let result = gaia.query(cmd)?;

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
    let gaia = GaiaNode::run()?;

    let cmd = GaiaTxCommands::Bank(BankTxCli {
        command: BankCommands::Send {
            to_address: AccAddress::from_bech32("cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut")?,
            amount: UnsignedCoin::from_str("10uatom")?,
        },
    });

    let responses = gaia
        .tx(cmd, GaiaNode::validator_key())?
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
        .decode("BC4739124707D9438CF490E6355B75E3038BD1D98BE787A950EB89B7A8A37CCA".as_bytes())?;

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
