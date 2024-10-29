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
use mint::types::query::{
    request::QueryAnnualProvisionsRequest, response::QueryAnnualProvisionsResponse,
};
use utils::set_node;

#[path = "./utils.rs"]
mod utils;

#[test]
fn query_provision_after_init() {
    let mut node = set_node();

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let q = QueryAnnualProvisionsRequest {};
    let ResponseQuery { value, .. } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryAnnualProvisionsRequest::QUERY_URL.to_owned(),
        height: 1,
        prove: false,
    });

    let QueryAnnualProvisionsResponse { annual_provisions } =
        QueryAnnualProvisionsResponse::decode_vec(&value).unwrap_test();

    let expected_provisions = Decimal256::zero();

    assert_eq!(expected_provisions, annual_provisions);
}

#[test]
fn query_provision_after_month_without_staking() {
    let mut node = set_node();

    let mut timestamp = Timestamp::UNIX_EPOCH;
    while timestamp.timestamp_seconds() <= TimestampSeconds::try_from(2_628_000).unwrap_test() {
        let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

        timestamp = timestamp
            .checked_add(Duration::new_from_secs(5))
            .unwrap_test();
    }

    let q = QueryAnnualProvisionsRequest {};
    let ResponseQuery { value, .. } = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryAnnualProvisionsRequest::QUERY_URL.to_owned(),
        height: 100, // todo
        prove: false,
    });

    let QueryAnnualProvisionsResponse { annual_provisions } =
        QueryAnnualProvisionsResponse::decode_vec(&value).unwrap_test();

    let expected_provisions = Decimal256::zero();

    assert_eq!(expected_provisions, annual_provisions);
}
