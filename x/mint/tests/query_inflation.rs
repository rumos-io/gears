use gears::{
    core::Protobuf,
    extensions::testing::UnwrapTesting,
    tendermint::types::{
        request::query::RequestQuery,
        response::ResponseQuery,
        time::{
            duration::Duration,
            timestamp::{Timestamp, TimestampSeconds},
        },
    },
    types::decimal256::Decimal256,
};
use mint::types::query::{request::QueryInflationRequest, response::QueryInflationResponse};
use utils::set_node;

#[path = "./utils.rs"]
mod utils;

#[test]
fn query_inflation_after_init() {
    let mut node = set_node();

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let q = QueryInflationRequest {};
    let ResponseQuery { value, .. } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryInflationRequest::QUERY_URL.to_owned(),
        height: 1,
        prove: false,
    });

    let QueryInflationResponse { inflation } =
        QueryInflationResponse::decode_vec(&value).unwrap_test();

    let expected_inflation = Decimal256::from_atomics(2_u8, 1).unwrap_test();

    assert_eq!(expected_inflation, inflation);
}

#[test]
fn query_inflation_after_month_without_staking() {
    let mut node = set_node();

    // Well. I simulate chain which runs for year and each block takes 5 seconds
    let mut timestamp = Timestamp::UNIX_EPOCH;
    while timestamp.timestamp_seconds() <= TimestampSeconds::try_from(2_628_000).unwrap_test() {
        let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

        timestamp = timestamp
            .checked_add(Duration::new_from_secs(5))
            .unwrap_test();
    }

    let q = QueryInflationRequest {};
    let ResponseQuery { value, .. } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryInflationRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    let QueryInflationResponse { inflation } =
        QueryInflationResponse::decode_vec(&value).unwrap_test();

    let expected_inflation = Decimal256::from_atomics(2_u8, 1).unwrap_test();

    assert_eq!(expected_inflation, inflation);
}
