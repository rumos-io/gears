use bank::types::query::{QueryBalanceRequest, QueryBalanceResponse};
use gears::core::Protobuf;
use gears::extensions::testing::UnwrapTesting;
use gears::tendermint::types::request::query::RequestQuery;
use gears::tendermint::types::time::timestamp::Timestamp;
use gears::types::base::coins::Coins;
use gears::types::msg::send::MsgSend;
use gears::utils::node::generate_tx;

use crate::setup_mock_node;

#[test]
/// In this scenario, we test the initialization of the application and submit a balance transfer on block three.
fn scenario_1() {
    let (mut node, user) = setup_mock_node(None::<&str>);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "76db0b7e8c2bbe8340719649cd7ceb18490a2789908634da1433f22493dcec5d"
    );

    node.step(vec![], Timestamp::UNIX_EPOCH);
    node.step(vec![], Timestamp::UNIX_EPOCH);

    let to_address = "cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut"
        .parse()
        .expect("hard coded address is valid");
    let amount = Coins::new(vec!["10uatom".parse().expect("hard coded coin is valid")])
        .expect("hard coded coins are valid");

    let msg = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: user.address(),
        to_address,
        amount,
    }));

    let txs = generate_tx(vec1::vec1![msg], 0, &user, node.chain_id().clone());

    let step_response = node.step(vec![txs], Timestamp::UNIX_EPOCH);

    assert_eq!(
        hex::encode(step_response.app_hash),
        "6276d7ba6271d98a204f58873797954a267bba87e3e9749fab630009a7df1c8d"
    );

    // check user balance
    let query = QueryBalanceRequest {
        address: user.address().into(),
        denom: "uatom".try_into().unwrap_test(),
    };
    let res = node.query(RequestQuery {
        data: query.encode_vec().into(),
        path: "/cosmos.bank.v1beta1.Query/Balance".to_string(),
        height: 0,
        prove: false,
    });
    let res = QueryBalanceResponse::decode(res.value).unwrap();
    assert_eq!(
        res.balance,
        Some("23uatom".parse().expect("hard coded coin is valid"))
    );
}
