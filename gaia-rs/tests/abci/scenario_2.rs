use std::path::Path;

use gears::{
    tendermint::types::{proto::crypto::PublicKey, time::timestamp::Timestamp},
    types::uint::Uint256,
};
use staking::{CommissionRates, CreateValidator, Description};

use crate::{setup_mock_node, User};

#[test]
/// This scenario has a richer genesis file, with more staking fields.
fn scenario_2() {
    let genesis_path = Path::new("./tests/abci/assets/scenario_2_genesis.json");
    let (mut node, _) = setup_mock_node(Some(genesis_path));
    let user = User::user_1(5);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "02f58b14fefe2bf8949b626bc7a7a3f870cb569a38424a3d026fad2203deb735"
    );

    //----------------------------------------
    // Create a validator

    let consensus_pub_key = serde_json::from_str::<PublicKey>(
        r#"{
    "type": "tendermint/PubKeyEd25519",
    "value": "NJWo4rSXCswNmK0Bttxzb8/1ioFNkRVi6Fio2KzAlCo="
    }"#,
    )
    .unwrap();

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::CreateValidator(CreateValidator {
            description: Description {
                moniker: "test".to_string(),
                identity: "".to_string(),
                website: "".to_string(),
                details: "".to_string(),
                security_contact: "".to_string(),
            },
            commission: CommissionRates::new(
                "0.1".parse().unwrap(),
                "1".parse().unwrap(),
                "0.1".parse().unwrap(),
            )
            .unwrap(),
            min_self_delegation: Uint256::from(100u32),
            delegator_address: user.address(),
            validator_address: user.address().into(),
            pub_key: consensus_pub_key,
            value: "10000uatom".parse().unwrap(),
        }));

    let txs = crate::generate_txs([(0, msg)], &user, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::try_new(60 * 60 * 24, 0).unwrap());
    assert_eq!(
        hex::encode(app_hash),
        "cf3f14e8812e97ab24cbe5bee1e8f50187fa9212992105d860c2c29b50c8fb70"
    );

    //----------------------------------------
    // Edit a validator - successfully

    let msg = gaia_rs::message::Message::Staking(staking::Message::EditValidator(
        staking::EditValidator::new(
            Description {
                moniker: "alice".to_string(),
                identity: "".to_string(),
                website: "".to_string(),
                security_contact: "".to_string(),
                details: "".to_string(),
            },
            Some("0.2".parse().unwrap()),
            Some(Uint256::from(200u32)),
            user.address().into(),
        ),
    ));

    let txs = crate::generate_txs([(1, msg)], &user, node.chain_id().clone());

    println!("txs: {:?}", txs[0].to_vec());
    // print hex encoded
    println!("txs: {:?}", hex::encode(txs[0].to_vec()));

    let app_hash = node.step(txs, Timestamp::try_new(60 * 60 * 24, 0).unwrap());
    assert_eq!(
        hex::encode(app_hash),
        "5fd02c41a162e9b14341e1192296b939f00aa2395997c6b64b2940e401f717df"
    );
}
