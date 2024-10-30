use gears::{
    core::Protobuf,
    extensions::testing::UnwrapTesting,
    tendermint::types::{
        request::query::RequestQuery, response::ResponseQuery, time::timestamp::Timestamp,
    },
    types::{decimal256::Decimal256, uint::Uint256},
    utils::node::{GenesisSource, User},
    x::types::validator::BondStatus,
};

use staking::{
    Commission, CommissionRates, Description, IbcV046Validator, QueryValidatorsRequest,
    QueryValidatorsResponse,
};
use utils::{set_node, USER_0};

#[path = "./utils.rs"]
mod utils;

#[test]
fn query_validators_empty() {
    let mut node = set_node(GenesisSource::Default);

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let q = QueryValidatorsRequest {
        status: BondStatus::Unspecified, // Query all validators
        pagination: None,
    };
    let ResponseQuery {
        code, value, log, ..
    } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorsRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    assert!(code == 0, "{log}");

    let QueryValidatorsResponse {
        validators,
        pagination: _,
    } = QueryValidatorsResponse::decode_vec(&value).unwrap_test();

    let expected_validators: Vec<IbcV046Validator> = vec![];

    pretty_assertions::assert_eq!(expected_validators, validators);
}

#[test]
fn query_validators_from_file() {
    let mut node = set_node(GenesisSource::File(
        "./tests/assets/query_validators.json".into(),
    ));

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let q = QueryValidatorsRequest {
        status: BondStatus::Unspecified, // Query all validators
        pagination: None,
    };
    let ResponseQuery {
        code, value, log, ..
    } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorsRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    assert!(code == 0, "{log}");

    let QueryValidatorsResponse {
        validators,
        pagination: _,
    } = QueryValidatorsResponse::decode_vec(&value).unwrap_test();

    let user = User::from_bech32(USER_0, 1).unwrap_test();

    let expected_validators: Vec<IbcV046Validator> = vec![IbcV046Validator {
        operator_address: user.address().into(),
        delegator_shares: Decimal256::from_atomics(5_u32, 0).unwrap_test(),
        description: Description::try_new("my_val", "", "", "", "").unwrap_test(),
        consensus_pubkey: serde_json::from_str("{\"@type\": \"/cosmos.crypto.ed25519.PubKey\", \"key\": \"6Ob7SEB++IzwqXQQ/pgsD/bkxXNl+LDBhJZwpKuvnMo=\"}").unwrap_test(),
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
    }];

    pretty_assertions::assert_eq!(expected_validators, validators);
}
