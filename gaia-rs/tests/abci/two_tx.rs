use gears::types::base::coins::Coins;
use gears::types::msg::send::MsgSend;
use gears::utils::node::generate_tx;
use gears::{tendermint::types::time::timestamp::Timestamp, types::address::AccAddress};

use crate::setup_mock_node;

#[test]
/// In this scenario, we test the initialization of the application and submit a balance transfer on block three then another on block four
fn two_tx_in_different_block() {
    let (mut node, user) = setup_mock_node(None::<&str>);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "76db0b7e8c2bbe8340719649cd7ceb18490a2789908634da1433f22493dcec5d"
    );

    node.step(vec![], Timestamp::UNIX_EPOCH);
    node.step(vec![], Timestamp::UNIX_EPOCH);

    let to_address: AccAddress = "cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut"
        .parse()
        .expect("hard coded address is valid");
    let amount = Coins::new(vec!["10uatom".parse().expect("hard coded coin is valid")])
        .expect("hard coded coins are valid");

    let msg = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: user.address(),
        to_address: to_address.to_owned(),
        amount: amount.to_owned(),
    }));

    let txs = generate_tx(vec1::vec1![msg], 0, &user, node.chain_id().clone());

    let app_hash = node.step(vec![txs], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "12c23374b049220003f26ede3dea6ca134a15072f895038cefe8c4a33108f4ae"
    );

    let msg = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: user.address(),
        to_address,
        amount,
    }));

    let txs = generate_tx(vec1::vec1![msg], 1, &user, node.chain_id().clone());

    let app_hash = node.step(vec![txs], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "fa38b70ea1cb45473a13bccf40d1cd836faa416f10ba5ce5f66b4db7e3224297"
    );
}

#[test]
/// In this scenario, we test the initialization of the application and submit a transaction on block three with two balance transfer messages
/// from the same account. This tests that the sequence number is incremented correctly.
fn two_tx_in_single_block() {
    let (mut node, user) = setup_mock_node(None::<&str>);

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "76db0b7e8c2bbe8340719649cd7ceb18490a2789908634da1433f22493dcec5d"
    );

    node.step(vec![], Timestamp::UNIX_EPOCH);
    node.step(vec![], Timestamp::UNIX_EPOCH);

    let to_address: AccAddress = "cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut"
        .parse()
        .expect("hard coded address is valid");
    let amount = Coins::new(vec!["10uatom".parse().expect("hard coded coin is valid")])
        .expect("hard coded coins are valid");

    let msg1 = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: user.address(),
        to_address: to_address.to_owned(),
        amount: amount.to_owned(),
    }));

    let msg2 = gaia_rs::message::Message::Bank(bank::Message::Send(MsgSend {
        from_address: user.address(),
        to_address,
        amount,
    }));

    let tx1 = generate_tx(vec1::vec1![msg1], 0, &user, node.chain_id().clone());
    let tx2 = generate_tx(vec1::vec1![msg2], 1, &user, node.chain_id().clone());

    let app_hash = node.step(vec![tx1, tx2], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        hex::encode(app_hash),
        "12c23374b049220003f26ede3dea6ca134a15072f895038cefe8c4a33108f4ae"
    );
}
