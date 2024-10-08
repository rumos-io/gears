use std::path::Path;

use gears::{
    tendermint::types::{proto::crypto::PublicKey, time::timestamp::Timestamp},
    types::uint::Uint256,
    utils::node::generate_tx,
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

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "e6e9ca71bd1f2c472018e02306c70b7058196e4211f62c93803a6e6c74922711"
    );

    //----------------------------------------
    // Create a validator

    let consensus_pub_key = serde_json::from_str::<PublicKey>(
        r#"{
    "type": "tendermint/PubKeyEd25519",
    "value": "NJWo4rSXCswNmK0Bttxzb8/1ioFNkRVi6Fio2KzAlCo="
    }"#,
    )
    .expect("hardcoded is valid");

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
                "0.1".parse().expect("hardcoded is valid"),
                "1".parse().expect("hardcoded is valid"),
                "0.1".parse().expect("hardcoded is valid"),
            )
            .expect("hardcoded is valid"),
            min_self_delegation: Uint256::from(100u32),
            delegator_address: user_1.address(),
            validator_address: user_1.address().into(),
            pubkey: consensus_pub_key,
            value: "10000uatom".parse().expect("hardcoded is valid"),
        }));

    let txs = generate_tx(vec1::vec1![msg], 0, &user_1, node.chain_id().clone());

    let app_hash = node
        .step(
            vec![txs],
            Timestamp::try_new(0, 0).expect("hardcoded is valid"),
        )
        .app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "6f02c4708c36481eeb65acf704340d906af5737702dbf05fc8bf4dd29a92f16e"
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
            Some("0.2".parse().expect("hardcoded is valid")),
            Some(Uint256::from(200u32)),
            user_1.address().into(),
        ),
    ));

    let txs = generate_tx(vec1::vec1![msg], 1, &user_1, node.chain_id().clone());

    let app_hash = node
        .step(
            vec![txs],
            Timestamp::try_new(60 * 60 * 24, 0).expect("hardcoded is valid"),
        )
        .app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "68f309714a2273b0f8ad93f318bc5a0dd418bd2bdd1431a6a848ae104c98a39b"
    );

    //----------------------------------------
    // Delegate to a validator

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Delegate(staking::DelegateMsg {
            validator_address: user_0.address().into(),
            amount: "1000uatom".parse().expect("hardcoded is valid"),
            delegator_address: user_1.address(),
        }));

    let txs = generate_tx(vec1::vec1![msg], 2, &user_1, node.chain_id().clone());

    let app_hash = node
        .step(
            vec![txs],
            Timestamp::try_new(60 * 60 * 24, 0).expect("hardcoded is valid"),
        )
        .app_hash;

    assert_eq!(
        hex::encode(app_hash),
        "2511608d208a0a99b5761f3820f8719ae71c6b67b577961caa993389d94985e7"
    );

    //----------------------------------------
    // Redelegate from a validator to another validator

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Redelegate(staking::RedelegateMsg {
            delegator_address: user_1.address(),
            src_validator_address: user_0.address().into(),
            dst_validator_address: user_1.address().into(),
            amount: "500uatom".parse().expect("hardcoded is valid"),
        }));

    let txs = generate_tx(vec1::vec1![msg], 3, &user_1, node.chain_id().clone());

    let app_hash = node
        .step(
            vec![txs],
            Timestamp::try_new(60 * 60 * 24, 0).expect("hardcoded is valid"),
        )
        .app_hash;

    assert_eq!(
        hex::encode(app_hash),
        "0cd0c37bfd4457d991ef5a694b3d903dc673a4836632147ba8ff1177c8d9632a"
    );

    //----------------------------------------
    // Undelegate from a validator

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Undelegate(staking::UndelegateMsg {
            validator_address: user_0.address().into(),
            amount: "500uatom".parse().expect("hardcoded is valid"),
            delegator_address: user_1.address(),
        }));

    let txs = generate_tx(vec1::vec1![msg], 4, &user_1, node.chain_id().clone());

    let app_hash = node
        .step(
            vec![txs],
            Timestamp::try_new(60 * 60 * 24, 0).expect("hardcoded is valid"),
        )
        .app_hash;

    assert_eq!(
        hex::encode(app_hash),
        "faf91423473800d139357be23c124a6d3b2919e9859e81d4b86a8c8f44d33b3d"
    );

    //----------------------------------------
    // Jump forward in time

    let app_hash = node
        .step(vec![], Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap())
        .app_hash; // 30 days which is greater than the unbonding time
    assert_eq!(
        hex::encode(app_hash),
        "3d8b7d855a8728d93c1a3918774cfdbf093b3263fdcdba1e1a0a9093466b43e9"
    );
}
