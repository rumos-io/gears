use std::path::Path;

use gears::core::Protobuf;
use gears::{
    tendermint::types::{
        proto::crypto::PublicKey, request::query::RequestQuery, time::timestamp::Timestamp,
    },
    types::uint::Uint256,
    utils::node::generate_tx,
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
    let user_0 = crate::user(3, USER_0);
    let user_1 = crate::user(4, USER_1);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "5f3f9b2a7b0a9d2d645da95bc8eed1c6a0105b52c36200ff68155cacfc4e5e08"
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

    let txs = generate_tx(vec1::vec1![msg], 1, &user_0, node.chain_id().clone());

    let step_res = node.step(vec![txs], Timestamp::try_new(0, 0).unwrap());

    assert!(
        step_res.tx_responses[0].log.contains("decode error: `error converting message type into domain type: error converting message type into domain type: decode error: `delegator address and validator address must be derived from the same public key"),
        // TODO: error messages are too verbose
    );
    assert_eq!(
        hex::encode(step_res.app_hash),
        "177683d0356ad00958fe7406f238e883850f1d0e1d815895fc56a813577167a7"
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

    let txs = generate_tx(vec1::vec1![msg], 0, &user_1, node.chain_id().clone());

    let step_response = node.step(
        vec![txs],
        Timestamp::try_new(0, 0).expect("hardcoded is valid"),
    );
    assert_eq!(
        hex::encode(step_response.app_hash),
        "d8e98a3173e681a3cb84f199c119080c5de2d33b6e837cffb27d779227c0e5b0"
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
        "885e9d12107d1c1bd3f6dc07d240a7cec4a2bfc5621d983d6d4070dbdfbc0152"
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

    let txs = generate_tx(vec1::vec1![msg], 1, &user_1, node.chain_id().clone());
    let step_response = node.step(vec![txs], Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap());

    assert_eq!(
        hex::encode(step_response.app_hash),
        "214d04fd965515cd12a2a1e97be0eeaff658f4e79cc6e65c47e9bdd78fc8650c"
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

    let txs = generate_tx(vec1::vec1![msg], 2, &user_1, node.chain_id().clone());

    let step_response = node.step(vec![txs], Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap());

    assert_eq!("transitive redelegation", step_response.tx_responses[0].log);

    assert_eq!(
        hex::encode(step_response.app_hash),
        "736ae35880680f418baa96627bde4daced7da324e12467ffa6617f593d9e756c"
    );

    //----------------------------------------
    // repeat redelegate from the bonded validator to the unbonded validator - this will test appending to the
    // redelegation_queue_time_slice

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Redelegate(staking::RedelegateMsg {
            delegator_address: user_1.address(),
            src_validator_address: user_1.address().into(),
            dst_validator_address: user_0.address().into(),
            amount: "4000000000uatom".parse().expect("hardcoded is valid"),
        }));

    let txs = generate_tx(vec1::vec1![msg], 3, &user_1, node.chain_id().clone());

    let step_response = node.step(vec![txs], Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap());

    assert_eq!(
        hex::encode(step_response.app_hash),
        "716c45a401203214af73502289602ab08783b36fff26f0e3ea7ccc63d3bf3ed6"
    );

    //----------------------------------------
    // delegate to user_1 - this should cause user_1 to go from unbonding to bonded

    // check user_1 is unbonding
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
    assert_eq!(res.validators[0].operator_address, user_1.address().into());

    // delegate to user_1
    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Delegate(staking::DelegateMsg {
            delegator_address: user_1.address(),
            validator_address: user_1.address().into(),
            amount: "31000000000uatom".parse().expect("hardcoded is valid"),
        }));

    let txs = generate_tx(vec1::vec1![msg], 4, &user_1, node.chain_id().clone());

    let step_response = node.step(vec![txs], Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap());

    assert_eq!(
        hex::encode(step_response.app_hash),
        "859f3457061b5e65492b20842f690abc00285554dc49bcd4c4fd37aab426ed19"
    );

    // check user_1 is bonded
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
    assert_eq!(res.validators[0].operator_address, user_1.address().into());

    //----------------------------------------
    // create two unbonding messages - this will check that we can read the unbonding queue

    let msg =
        gaia_rs::message::Message::Staking(staking::Message::Undelegate(staking::UndelegateMsg {
            validator_address: user_1.address().into(),
            amount: "1000000000uatom".parse().expect("hardcoded is valid"),
            delegator_address: user_1.address(),
        }));

    let txs = generate_tx(
        vec1::vec1![msg.clone(), msg],
        5,
        &user_1,
        node.chain_id().clone(),
    );

    let step_response = node.step(vec![txs], Timestamp::try_new(60 * 60 * 24 * 30, 0).unwrap());

    assert_eq!(
        hex::encode(step_response.app_hash),
        "7f0646e9c8b2a12235088c6cdfaedcd0ea67fb4957112e51a85b8d2f07915933"
    );
}
