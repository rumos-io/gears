use gears::{
    core::Protobuf,
    extensions::testing::UnwrapTesting,
    tendermint::types::{
        request::query::RequestQuery, response::ResponseQuery, time::timestamp::Timestamp,
    },
    utils::node::GenesisSource,
    x::types::validator::BondStatus,
};

use staking::{IbcV046Validator, QueryValidatorsRequest, QueryValidatorsResponse};
use utils::set_node;

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
    let ResponseQuery { code, value, .. } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorsRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    assert!(code == 0);

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
        "./tests/assets/genesis_with_validator.json".into(),
    ));

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let q = QueryValidatorsRequest {
        status: BondStatus::Unspecified, // Query all validators
        pagination: None,
    };
    let ResponseQuery { code, value, .. } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorsRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    assert!(code == 0);

    let QueryValidatorsResponse {
        validators,
        pagination: _,
    } = QueryValidatorsResponse::decode_vec(&value).unwrap_test();

    let expected_validators: Vec<IbcV046Validator> = vec![];

    pretty_assertions::assert_eq!(expected_validators, validators);
}
