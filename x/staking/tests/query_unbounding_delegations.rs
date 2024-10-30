use gears::{
    core::Protobuf,
    extensions::testing::UnwrapTesting,
    tendermint::types::{
        request::query::RequestQuery, response::ResponseQuery, time::timestamp::Timestamp,
    },
    utils::node::{GenesisSource, User},
};
use staking::{
    QueryValidatorUnbondingDelegationsRequest, QueryValidatorUnbondingDelegationsResponse,
    UnbondingDelegation,
};
use utils::{set_node, USER_0};

#[path = "./utils.rs"]
mod utils;

#[test]
fn query_unbounding_delegations_empty() {
    let mut node = set_node(GenesisSource::Default);

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let user = User::from_bech32(USER_0, 1).unwrap_test();

    let q = QueryValidatorUnbondingDelegationsRequest {
        validator_addr: user.address().into(),
        pagination: None,
    };
    let ResponseQuery {
        code, value, log, ..
    } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorUnbondingDelegationsRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    assert!(code == 0, "{log}");

    let QueryValidatorUnbondingDelegationsResponse {
        unbonding_responses,
        pagination: _,
    } = QueryValidatorUnbondingDelegationsResponse::decode_vec(&value).unwrap_test();

    let expected_responses: Vec<UnbondingDelegation> = vec![];

    pretty_assertions::assert_eq!(expected_responses, unbonding_responses);
}
