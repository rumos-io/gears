use gears::tendermint::types::time::timestamp::Timestamp;
use gears::types::base::coins::Coins;
use gears::types::msg::send::MsgSend;

use crate::{generate_txs, setup_mock_node};

#[test]
/// In this scenario, we test the initialization of the application and submit a balance transfer on block three.
fn scenario_1() {
    let (mut node, user) = setup_mock_node();

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "d0254da38fc9c97292f65f4e8af3276209c6d6f8a922bbad8fc4a8f36af55f67"
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
        "7bc0e95da6ba637bddaade5e6911fdb20030956a4bb8e305fb1c390ff7bcea20"
    );
}
