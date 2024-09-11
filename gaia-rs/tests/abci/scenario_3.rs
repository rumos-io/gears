use std::path::Path;

use gears::tendermint::types::time::timestamp::Timestamp;

use crate::setup_mock_node;

#[test]
/// This scenario's genesis includes a gentx.
fn scenario_3() {
    let genesis_path = Path::new("./tests/abci/assets/scenario_3_genesis.json");
    let (mut node, _) = setup_mock_node(Some(genesis_path));

    let app_hash = node.step(vec![], Timestamp::UNIX_EPOCH);
    assert_eq!(
        hex::encode(app_hash),
        "47cea41289131655a4843a77f02b551d1f91e155d73826aecb8062de80ec8b75"
    );
}
