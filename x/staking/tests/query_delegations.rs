use std::str::FromStr;

use gears::{
    core::Protobuf,
    extensions::testing::UnwrapTesting,
    tendermint::types::{
        request::query::RequestQuery, response::ResponseQuery, time::timestamp::Timestamp,
    },
    types::{base::coin::UnsignedCoin, decimal256::Decimal256},
    utils::node::{GenesisSource, User},
};
use staking::{
    Delegation, DelegationResponse, QueryValidatorDelegationsRequest,
    QueryValidatorDelegationsResponse,
};
use utils::{set_node, USER_0};

#[path = "./utils.rs"]
mod utils;

#[test]
fn query_delegations_empty() {
    let mut node = set_node(GenesisSource::Default);

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let user = User::from_bech32(USER_0, 1).unwrap_test();

    let q = QueryValidatorDelegationsRequest {
        validator_addr: user.address().into(),
        pagination: None,
    };
    let ResponseQuery {
        code, value, log, ..
    } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorDelegationsRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    assert!(code == 0, "{log}");

    let QueryValidatorDelegationsResponse {
        delegation_responses,
        pagination: _,
    } = QueryValidatorDelegationsResponse::decode_vec(&value).unwrap_test();

    let expected_responses: Vec<DelegationResponse> = vec![];

    pretty_assertions::assert_eq!(expected_responses, delegation_responses);
}

#[test]
fn query_delegations() {
    let mut node = set_node(GenesisSource::File(
        "./tests/assets/query_validators.json".into(),
    ));

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let user = User::from_bech32(USER_0, 1).unwrap_test();

    let q = QueryValidatorDelegationsRequest {
        validator_addr: user.address().into(),
        pagination: None,
    };
    let ResponseQuery {
        code, value, log, ..
    } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorDelegationsRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    assert!(code == 0, "{log}");

    let QueryValidatorDelegationsResponse {
        delegation_responses,
        pagination: _,
    } = QueryValidatorDelegationsResponse::decode_vec(&value).unwrap_test();

    let expected_responses: Vec<DelegationResponse> = vec![DelegationResponse {
        delegation: Some(Delegation {
            delegator_address: user.address(),
            validator_address: user.address().into(),
            shares: Decimal256::from_atomics(1000000000000000000_u64, 0).unwrap_test(),
        }),
        balance: Some(UnsignedCoin::from_str("1000000000000000000uatom").unwrap_test()),
    }];

    pretty_assertions::assert_eq!(expected_responses, delegation_responses);
}
