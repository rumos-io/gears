use std::path::Path;

use gears::core::Protobuf;
use gears::{
    tendermint::types::{
        proto::crypto::PublicKey, request::query::RequestQuery, time::timestamp::Timestamp,
    },
    types::uint::Uint256,
    utils::node::generate_txs,
    x::types::validator::BondStatus,
};
use staking::{
    CommissionRates, CreateValidator, Description, QueryValidatorsRequest, QueryValidatorsResponse,
};

use crate::{setup_mock_node, USER_0, USER_1};

#[test]
/// This scenario's genesis includes a gentx.
fn scenario_3() {
    let genesis_path = Path::new("./tests/abci/assets/scenario_3_genesis.json");
    let (mut node, _) = setup_mock_node(Some(genesis_path));
    let user_0 = crate::user(2, USER_0);
    let user_1 = crate::user(3, USER_1);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "e111f4a62a52f13c7e942694aa9c6997f4f6e131b9306090aa022297ce362540"
    );

    //----------------------------------------
    // Try to create a validator with validator address and delegator address derived from different keys - should fail

    let consensus_pub_key = serde_json::from_str::<PublicKey>(
        r#"{
    "type": "tendermint/PubKeyEd25519",
    "value": "AFn3B2/Dvyu9csqfifLNiW1B+D8FvcabD5NW+fGZLPc="
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
            delegator_address: user_0.address(),
            validator_address: user_1.address().into(),
            pubkey: consensus_pub_key,
            value: "10000uatom".parse().unwrap(),
        }));

    let txs = generate_txs([(1, msg)], &user_0, node.chain_id().clone());

    let step_res = node.step(txs, Timestamp::try_new(0, 0).unwrap());

    assert_eq!(
        step_res.tx_responses[0].log,
        // TODO: error messages are too verbose
        "decode error: `error converting message type into domain type: error converting message type into domain type: decode error: `delegator address and validator address must be derived from the same public key`\n\nLocation:\n    /home/kevin/.cargo/registry/src/index.crates.io-6f17d22bba15001f/flex-error-0.4.4/src/tracer_impl/eyre.rs:10:9\n\nLocation:\n    /home/kevin/.cargo/registry/src/index.crates.io-6f17d22bba15001f/flex-error-0.4.4/src/tracer_impl/eyre.rs:10:9`".to_string()
    );
    assert_eq!(
        hex::encode(step_res.app_hash),
        "6d0b1e5f3f4f3759c05be2eabed1f4586d176ab36f76df7d9b874dbe850016c8"
    );

    // query the validator list
    let query = QueryValidatorsRequest {
        status: BondStatus::Bonded,
        pagination: None,
    };

    let res = node.query(RequestQuery {
        data: query.encode_vec().into(),
        path: "/cosmos.staking.v1beta1.Query/Validators".to_string(),
        height: 0,
        prove: false,
    });

    let res = QueryValidatorsResponse::decode(res.value).unwrap();
    assert_eq!(res.validators.len(), 1);

    //----------------------------------------
    // Create a validator with more voting power than the one in the genesis file - should cause the genesis validator to be unbonded

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
            value: "20000000000uatom".parse().expect("hardcoded is valid"),
        }));

    let txs = generate_txs([(0, msg)], &user_1, node.chain_id().clone());

    let app_hash = node
        .step(txs, Timestamp::try_new(0, 0).expect("hardcoded is valid"))
        .app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "815b88380e50eb8a82f9df53503dddb14cba409970aaf77e7de1164ca8bc61f5"
    );

    // query the validator list
    let query = QueryValidatorsRequest {
        status: BondStatus::Unbonding,
        pagination: None,
    };

    let res = node.query(RequestQuery {
        data: query.encode_vec().into(),
        path: "/cosmos.staking.v1beta1.Query/Validators".to_string(),
        height: 0,
        prove: false,
    });

    let res = QueryValidatorsResponse::decode(res.value).unwrap();
    assert_eq!(res.validators.len(), 1);
    assert_eq!(res.validators[0].operator_address, user_0.address().into());

    //----------------------------------------
    // Jump forward in time - the unbonding validator will be unbonded

    let app_hash = node
        .step(vec![], Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap())
        .app_hash; // 30 days which is greater than the unbonding time
    assert_eq!(
        hex::encode(app_hash),
        "07f42dc05073c352627503e52acd89538ddcf08a0bb7d385027938f32013cc1e"
    );

    //----------------------------------------
    // redelegate from the bonded validator to the unbonded validator - we want to create an unbonding validator (user_1)

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Redelegate(staking::RedelegateMsg {
            delegator_address: user_1.address(),
            src_validator_address: user_1.address().into(),
            dst_validator_address: user_0.address().into(),
            amount: "15000000000uatom".parse().expect("hardcoded is valid"),
        }));

    let txs = generate_txs([(1, msg)], &user_1, node.chain_id().clone());

    let step_response = node.step(txs, Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap());

    assert_eq!(
        hex::encode(step_response.app_hash),
        "9c4df3dd21c2eee54b0ac4a831615fae64417cea9d2d98301c6a2b0ce63c2963"
    );

    //----------------------------------------
    // try to revert the previous redelegation - should fail (transitive redelegations are not allowed)

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Redelegate(staking::RedelegateMsg {
            delegator_address: user_1.address(),
            src_validator_address: user_0.address().into(),
            dst_validator_address: user_1.address().into(),
            amount: "15000000000uatom".parse().expect("hardcoded is valid"),
        }));

    let txs = generate_txs([(2, msg)], &user_1, node.chain_id().clone());

    let step_response = node.step(txs, Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap());

    assert_eq!("transitive redelegation", step_response.tx_responses[0].log);

    assert_eq!(
        hex::encode(step_response.app_hash),
        "c60067ca69637b341607ed8bbf19048b5b04b6860dbe9f01f91b0d09d18c9e8e"
    );

    //----------------------------------------
    // repeat redelegate from the bonded validator to the unbonded validator - this will test appending to the
    // redelegation_queue_time_slice

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Redelegate(staking::RedelegateMsg {
            delegator_address: user_1.address(),
            src_validator_address: user_1.address().into(),
            dst_validator_address: user_0.address().into(),
            amount: "5000000000uatom".parse().expect("hardcoded is valid"),
        }));

    let txs = generate_txs([(3, msg)], &user_1, node.chain_id().clone());

    println!("txs: {:?}", txs[0].to_vec());

    let step_response = node.step(txs, Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap());

    assert_eq!(
        hex::encode(step_response.app_hash),
        "89584a1350b27aa49cba197b36550a98a2ecc2c5e25fd0807f1c16ae73f26d6f"
    );
}
