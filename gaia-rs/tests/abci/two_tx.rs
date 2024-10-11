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
        "fadd423b82c26b2a22effdd4b99e8c685dd9c7281eab3d1911b303599a98dd75"
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
        "c87125d99593ca696ce60c2195ef67125fb434714c36c4b4d1fe1d1eb4f1580e"
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
        "799fe6e722efd5582207c548be705472983b10ab19d48db98512569931a85eaa"
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
        "fadd423b82c26b2a22effdd4b99e8c685dd9c7281eab3d1911b303599a98dd75"
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
        "b0de8e449befc5ad60b10cabcbd4dd184c025b4697922329d26b87facd92b8e2"
    );
}
