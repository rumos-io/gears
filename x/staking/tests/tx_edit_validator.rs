use gears::{
    core::Protobuf,
    extensions::testing::UnwrapTesting,
    tendermint::types::{
        request::query::RequestQuery, response::ResponseDeliverTx, time::timestamp::Timestamp,
    },
    types::{decimal256::Decimal256, uint::Uint256},
    utils::node::{generate_tx, GenesisSource, StepResponse, User},
    x::types::validator::BondStatus,
};
use staking::{
    Commission, CommissionRates, Description, EditDescription, EditValidator, IbcV046Validator,
    Message, QueryValidatorRequest, QueryValidatorResponse,
};
use utils::{set_node, USER_0};

#[path = "./utils.rs"]
mod utils;

const GENESIS_FILE_PATH: &str = "./tests/assets/tx_edit_validator.json";

#[test]
fn edit_validator_unbounded_no_changes() {
    let mut node = set_node(GenesisSource::File(GENESIS_FILE_PATH.into()));

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let user = User::from_bech32(USER_0, 1).unwrap_test();

    let msg = EditValidator::new(
        EditDescription::default(),
        None,
        None,
        user.address().into(),
    );

    let txs = generate_tx(
        vec1::vec1![Message::EditValidator(msg)],
        0,
        &user,
        node.chain_id().clone(),
    );

    let StepResponse {
        app_hash,
        mut tx_responses,
        height: _,
    } = node.step(vec![txs], Timestamp::UNIX_EPOCH);

    let ResponseDeliverTx { code, log, .. } = tx_responses.pop().unwrap_test();

    assert!(code == 0, "{log}");

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "db9f34b4442891046703b4e2a9ddfecd25dba95e79f3d92d26eae04c05698a33"
    );

    let StepResponse {
        app_hash,
        tx_responses: _,
        height: _,
    } = node.step(vec![], Timestamp::UNIX_EPOCH);

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "b75c58a5cb592fa900dd48feeaae5db42f1480a6eb28441cfc54ad6a981e0b33"
    );

    let q = QueryValidatorRequest {
        validator_addr: user.address().into(),
    };
    let query_result = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    let validator = match QueryValidatorResponse::decode(query_result.value)
        .unwrap_test()
        .validator
    {
        Some(deposit) => deposit,
        None => panic!("failed to find validator"),
    };

    let expected_validator = IbcV046Validator {
        operator_address: user.address().into(),
        delegator_shares: Decimal256::from_atomics(5_u32, 0).unwrap_test(),
        description: Description::try_new("my_val", "", "", "", "").unwrap_test(),
        consensus_pubkey:  serde_json::from_str("{\"@type\": \"/cosmos.crypto.ed25519.PubKey\", \"key\": \"6Ob7SEB++IzwqXQQ/pgsD/bkxXNl+LDBhJZwpKuvnMo=\"}").unwrap_test(),
        jailed: false,
        tokens: Uint256::from(5_u32),
        unbonding_height: 1,
        unbonding_time: Timestamp::try_new(1814400, 0).unwrap_test(),
        commission: Commission::new(
            CommissionRates::new(
                Decimal256::from_atomics(1u64, 1).unwrap_test(),
                Decimal256::from_atomics(2u64, 1).unwrap_test(),
                Decimal256::from_atomics(1u64, 1).unwrap_test(),
            )
            .unwrap_test(),
            Timestamp::try_new(1722359411, 32635319).unwrap_test(),
        ),
        min_self_delegation: Uint256::one(),
        status: BondStatus::Unbonding,
        unbonding_ids: Vec::new(),
        unbonding_on_hold_ref_count: Uint256::zero(),
        validator_bond_shares: Decimal256::zero(),
        liquid_shares: Decimal256::zero(),
    };

    pretty_assertions::assert_eq!(expected_validator, validator);
}

#[test]
fn edit_validator_unbounded_edit_desc() {
    let mut node = set_node(GenesisSource::File(GENESIS_FILE_PATH.into()));

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let user = User::from_bech32(USER_0, 1).unwrap_test();

    let msg = EditValidator::new(
        EditDescription {
            moniker: Some("new".to_owned()),
            identity: Some("identity".to_owned()),
            website: Some("website".to_owned()),
            security_contact: Some("security_contact".to_owned()),
            details: Some("some great details about this great validator".to_owned()),
        },
        None,
        None,
        user.address().into(),
    );

    let txs = generate_tx(
        vec1::vec1![Message::EditValidator(msg)],
        0,
        &user,
        node.chain_id().clone(),
    );

    let StepResponse {
        app_hash,
        mut tx_responses,
        height: _,
    } = node.step(vec![txs], Timestamp::UNIX_EPOCH);

    let ResponseDeliverTx { code, log, .. } = tx_responses.pop().unwrap_test();

    assert!(code == 0, "{log}");

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "030c5c2ffb9c1111d802fba413bd75c0e21efb0c528a66875729b387966e4c30"
    );

    let StepResponse {
        app_hash,
        tx_responses: _,
        height: _,
    } = node.step(vec![], Timestamp::UNIX_EPOCH);

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "445f3071bc78383c06d91a887e097d83491df3d6417378a2c021e1dbb238a20b"
    );

    let q = QueryValidatorRequest {
        validator_addr: user.address().into(),
    };
    let query_result = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    let validator = match QueryValidatorResponse::decode(query_result.value)
        .unwrap_test()
        .validator
    {
        Some(deposit) => deposit,
        None => panic!("failed to find validator"),
    };

    let expected_validator = IbcV046Validator {
        operator_address: user.address().into(),
        delegator_shares: Decimal256::from_atomics(5_u32, 0).unwrap_test(),
        description: Description::try_new("new", "identity", "website", "security_contact", "some great details about this great validator").unwrap_test(),
        consensus_pubkey:  serde_json::from_str("{\"@type\": \"/cosmos.crypto.ed25519.PubKey\", \"key\": \"6Ob7SEB++IzwqXQQ/pgsD/bkxXNl+LDBhJZwpKuvnMo=\"}").unwrap_test(),
        jailed: false,
        tokens: Uint256::from(5_u32),
        unbonding_height: 1,
        unbonding_time: Timestamp::try_new(1814400, 0).unwrap_test(),
        commission: Commission::new(
            CommissionRates::new(
                Decimal256::from_atomics(1u64, 1).unwrap_test(),
                Decimal256::from_atomics(2u64, 1).unwrap_test(),
                Decimal256::from_atomics(1u64, 1).unwrap_test(),
            )
            .unwrap_test(),
            Timestamp::try_new(1722359411, 32635319).unwrap_test(),
        ),
        min_self_delegation: Uint256::one(),
        status: BondStatus::Unbonding,
        unbonding_ids: Vec::new(),
        unbonding_on_hold_ref_count: Uint256::zero(),
        validator_bond_shares: Decimal256::zero(),
        liquid_shares: Decimal256::zero(),
    };

    pretty_assertions::assert_eq!(expected_validator, validator);
}

// #[test]
// #[should_panic]
// fn edit_validator_unbounded_edit_min_self_delegation_fails() {
//     let mut node = set_node(GenesisSource::File(GENESIS_FILE_PATH.into()));

//     let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

//     let user = User::from_bech32(USER_0, 1).unwrap_test();

//     let msg = EditValidator::new(
//         EditDescription::default(),
//         None,
//         Some(Uint256::from(u64::MAX)),
//         user.address().into(),
//     );

//     let txs = generate_tx(
//         vec1::vec1![Message::EditValidator(msg)],
//         0,
//         &user,
//         node.chain_id().clone(),
//     );
//     let _ = node.step(vec![txs], Timestamp::UNIX_EPOCH);
// }

#[test]
fn edit_validator_unbounded_edit_min_self_delegation() {
    let mut node = set_node(GenesisSource::File(GENESIS_FILE_PATH.into()));

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let user = User::from_bech32(USER_0, 1).unwrap_test();

    let msg = EditValidator::new(
        EditDescription::default(),
        None,
        Some(Uint256::from(5_u32)),
        user.address().into(),
    );

    let txs = generate_tx(
        vec1::vec1![Message::EditValidator(msg)],
        0,
        &user,
        node.chain_id().clone(),
    );

    let StepResponse {
        app_hash,
        mut tx_responses,
        height: _,
    } = node.step(vec![txs], Timestamp::UNIX_EPOCH);

    let ResponseDeliverTx { code, log, .. } = tx_responses.pop().unwrap_test();

    assert!(code == 0, "{log}");

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "689a2839aba8d44a884908fd8142a54d209755f94de87483b6a201cc9c603a77"
    );

    let StepResponse {
        app_hash,
        tx_responses: _,
        height: _,
    } = node.step(vec![], Timestamp::UNIX_EPOCH);

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "ea3750c092b285382db8902959fa3ced90a40c281fad51fcaf1ca47177ae6712"
    );

    let q = QueryValidatorRequest {
        validator_addr: user.address().into(),
    };
    let query_result = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    let validator = match QueryValidatorResponse::decode(query_result.value)
        .unwrap_test()
        .validator
    {
        Some(deposit) => deposit,
        None => panic!("failed to find validator"),
    };

    let expected_validator = IbcV046Validator {
        operator_address: user.address().into(),
        delegator_shares: Decimal256::from_atomics(5_u32, 0).unwrap_test(),
        description: Description::try_new("my_val", "", "", "", "").unwrap_test(),
        consensus_pubkey:  serde_json::from_str("{\"@type\": \"/cosmos.crypto.ed25519.PubKey\", \"key\": \"6Ob7SEB++IzwqXQQ/pgsD/bkxXNl+LDBhJZwpKuvnMo=\"}").unwrap_test(),
        jailed: false,
        tokens: Uint256::from(5_u32),
        unbonding_height: 1,
        unbonding_time: Timestamp::try_new(1814400, 0).unwrap_test(),
        commission: Commission::new(
            CommissionRates::new(
                Decimal256::from_atomics(1u64, 1).unwrap_test(),
                Decimal256::from_atomics(2u64, 1).unwrap_test(),
                Decimal256::from_atomics(1u64, 1).unwrap_test(),
            )
            .unwrap_test(),
            Timestamp::try_new(1722359411, 32635319).unwrap_test(),
        ),
        min_self_delegation: Uint256::from(5_u8),
        status: BondStatus::Unbonding,
        unbonding_ids: Vec::new(),
        unbonding_on_hold_ref_count: Uint256::zero(),
        validator_bond_shares: Decimal256::zero(),
        liquid_shares: Decimal256::zero(),
    };

    pretty_assertions::assert_eq!(expected_validator, validator);
}
