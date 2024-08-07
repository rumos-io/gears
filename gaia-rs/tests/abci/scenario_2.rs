use std::path::Path;

use gears::tendermint::types::time::timestamp::Timestamp;

use crate::setup_mock_node;

#[test]
/// In this scenario, we test the initialization of the application and submit a balance transfer on block three.
fn scenario_2() {
    let genesis_path = Path::new("./tests/abci/assets/scenario_2_genesis.json");
    let (mut node, _) = setup_mock_node(Some(genesis_path));

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "819c6ebd1fabc67ea71e2ff0d60081c1ea6ffee88d401d74a6e0c63cc1e9c32b"
    );
}
