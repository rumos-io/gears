#![cfg(feature = "it")]

use std::str::FromStr;

use bank::cli::tx::{BankCommands, BankTxCli};
use gaia_rs::client::GaiaTxCommands;
use gears::{
    tendermint::{
        abci::{Event, EventAttribute},
        rpc::response::tx::broadcast::Response,
    },
    types::{address::AccAddress, base::coin::UnsignedCoin},
};
use utilities::GaiaNode;

#[path = "./utilities.rs"]
mod utilities;

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

    let expected_hash =
        "0A2E6111D1B81ED9F6D40F0A27A2A36374D361BC455DB8E489E4697B37E7E1E1".to_string();

    assert_eq!(expected_hash, hash.to_string());
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
