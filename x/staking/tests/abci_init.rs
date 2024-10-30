use gears::{tendermint::types::time::timestamp::Timestamp, utils::node::GenesisSource};

use utils::set_node;

#[path = "./utils.rs"]
mod utils;

#[test]
/// In this scenario, we test the initialization of the application and execute a few blocks
fn test_init_and_few_blocks() {
    let mut node = set_node(GenesisSource::Default);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "d37be76009c3aedba96db9b8f623b5bd23f1e4802ae4fbca54e91728c6376e14"
    );

    node.skip_steps(100);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "ca4af6227631cdc873bf61b11815153dde9ef883d7c829ba1a63121e32dcf213"
    );
}
