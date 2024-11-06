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
        "9a6bf6c50ecff19e4ea4838630b5adb8d399a05b62ff742dc26d316f019e54ca"
    );

    node.skip_steps(100);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "a31893c018dc6bb7cf756c57a2e0e252fdbbf83b33e3039307ade8476dbef999"
    );
}
