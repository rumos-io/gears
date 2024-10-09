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
    let user_0 = crate::user(5, USER_0);
    let user_1 = crate::user(6, USER_1);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "87308fa82b6de74ed14ac1e701aeb66e7844494ba1713c37328cca3ce7884bf0"
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

    let step_response = node.step(
        vec![txs],
        Timestamp::try_new(0, 0).expect("hardcoded is valid"),
    );
    assert_eq!(step_response.tx_responses[0].code, 0);
    assert_eq!(
        hex::encode(step_response.app_hash),
        "7134aa05a768927784a369aecdc42bd1c77c01c41e6bbf861cb52edefd2f6ff5"
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

    let step_response = node.step(
        vec![txs],
        Timestamp::try_new(60 * 60 * 24, 0).expect("hardcoded is valid"),
    );
    assert_eq!(step_response.tx_responses[0].code, 0);
    assert_eq!(
        hex::encode(step_response.app_hash),
        "661cf36acf893031c237cb8381845211fa8ee59160e4e0bd0c86f56e17d8f0d5"
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

    let step_response = node.step(
        vec![txs],
        Timestamp::try_new(60 * 60 * 24, 0).expect("hardcoded is valid"),
    );
    assert_eq!(step_response.tx_responses[0].code, 0);

    assert_eq!(
        hex::encode(step_response.app_hash),
        "86995fd7296fa474c6b2fcead1ac40b36b5b69e03e97bd47e0549157719535f0"
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

    let step_response = node.step(
        vec![txs],
        Timestamp::try_new(60 * 60 * 24, 0).expect("hardcoded is valid"),
    );
    assert_eq!(step_response.tx_responses[0].code, 0);

    assert_eq!(
        hex::encode(step_response.app_hash),
        "02347e5bb6d59518bed1c73cbbbf22025c242f01332d5101aa1310080dfe243a"
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

    let step_response = node.step(
        vec![txs],
        Timestamp::try_new(60 * 60 * 24, 0).expect("hardcoded is valid"),
    );
    assert_eq!(step_response.tx_responses[0].code, 0);

    assert_eq!(
        hex::encode(step_response.app_hash),
        "9ebdae93785523a55178ede198f70adfbc86126b7fffda4f95c784a2f8cd2960"
    );

    //----------------------------------------
    // Jump forward in time

    let app_hash = node
        .step(vec![], Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap())
        .app_hash; // 30 days which is greater than the unbonding time
    assert_eq!(
        hex::encode(app_hash),
        "ef1d229776930752d005278738ea2ceab02dc184e4a0795e4f8dd503bc2a87d6"
    );
}
