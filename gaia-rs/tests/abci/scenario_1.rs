use gears::tendermint::types::time::timestamp::Timestamp;
use gears::types::base::coins::Coins;
use gears::types::msg::send::MsgSend;
use gears::utils::node::generate_txs;

use crate::setup_mock_node;

#[test]
/// In this scenario, we test the initialization of the application and submit a balance transfer on block three.
fn scenario_1() {
    let (mut node, user) = setup_mock_node(None::<&str>);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "36fd98b5248f0e4bfa6ef4e311134403b1b3deb8865bdbba7187cf05e5644a83"
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

    let txs = generate_txs([(0, msg)], &user, node.chain_id().clone());

    let app_hash = node.step(txs, Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "8eb5f41a3f77e034185be06e5385ff0d0a42f8d0f59171b1cc12b1ac6a66bbef"
    );
}
