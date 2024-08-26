use std::path::Path;

use gears::{
    tendermint::types::{proto::crypto::PublicKey, time::timestamp::Timestamp},
    types::uint::Uint256,
    utils::node::generate_txs,
};
use staking::{CommissionRates, CreateValidator, Description, EditDescription};

use crate::{setup_mock_node, USER_0, USER_1};

#[test]
/// This scenario has a richer genesis file, with more staking fields.
fn scenario_2() {
    let genesis_path = Path::new("./tests/abci/assets/scenario_2_genesis.json");
    let (mut node, _) = setup_mock_node(Some(genesis_path));
    let user_0 = crate::user(4, USER_0);
    let user_1 = crate::user(5, USER_1);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "3de46dd2cfc6c80718b3c70149ef0faf0a69e1f56a2cd82336521288092e47ad"
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
            delegator_address: user_1.address(),
            validator_address: user_1.address().into(),
            pubkey: consensus_pub_key,
            value: "10000uatom".parse().unwrap(),
        }));

    let txs = generate_txs([(0, msg)], &user_1, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::try_new(0, 0).unwrap());
    assert_eq!(
        hex::encode(app_hash),
        "d5170c89bd5de8774bc0d596a341fe422152a0407289610c27b904ba61b8ca26"
    );

    //----------------------------------------
    // Edit a validator - successfully

    let msg = gaia_rs::message::Message::Staking(staking::Message::EditValidator(
        staking::EditValidator::new(
            EditDescription {
                moniker: Some("alice".to_string()),
                identity: Some("".to_string()),
                website: Some("".to_string()),
                security_contact: Some("".to_string()),
                details: Some("".to_string()),
            },
            Some("0.2".parse().unwrap()),
            Some(Uint256::from(200u32)),
            user_1.address().into(),
        ),
    ));

    let txs = generate_txs([(1, msg)], &user_1, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::try_new(60 * 60 * 24, 0).unwrap());
    assert_eq!(
        hex::encode(app_hash),
        "323c2ba83da0cd456c2406365259a8783e7fee8e726324faea32f3d1d5c5003c"
    );

    //----------------------------------------
    // Delegate to a validator

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Delegate(staking::DelegateMsg {
            validator_address: user_0.address().into(),
            amount: "1000uatom".parse().unwrap(),
            delegator_address: user_1.address(),
        }));

    let txs = generate_txs([(2, msg)], &user_1, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::try_new(60 * 60 * 24, 0).unwrap());

    assert_eq!(
        hex::encode(app_hash),
        "ddea07dad9c7ea9430c4bc1041e4a890f2c4e21d615362f961d45501b9440539"
    );

    //----------------------------------------
    // Redelegate from a validator to another validator

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Redelegate(staking::RedelegateMsg {
            delegator_address: user_1.address(),
            src_validator_address: user_0.address().into(),
            dst_validator_address: user_1.address().into(),
            amount: "500uatom".parse().unwrap(),
        }));

    let txs = generate_txs([(3, msg)], &user_1, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::try_new(60 * 60 * 24, 0).unwrap());

    assert_eq!(
        hex::encode(app_hash),
        "3ff5e4bf5583eece172e9235cc4655c256cff8d8ccb60656b0808bb1224ccd27"
    );

    //----------------------------------------
    // Undelegate from a validator

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Undelegate(staking::UndelegateMsg {
            validator_address: user_0.address().into(),
            amount: "500uatom".parse().unwrap(),
            delegator_address: user_1.address(),
        }));

    let txs = generate_txs([(4, msg)], &user_1, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::try_new(60 * 60 * 24, 0).unwrap());

    assert_eq!(
        hex::encode(app_hash),
        "3582cbcbf99cb8b7748d24eac62d009746bd779ac2913c8af1ad3b2c4b22c945"
    );
}
