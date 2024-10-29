use gears::tendermint::types::time::timestamp::Timestamp;
use utils::set_node;

#[path = "./utils.rs"]
mod utils;

#[test]
fn test_init_and_few_blocks() {
    let mut node = set_node();

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;

    assert_eq!(data_encoding::HEXLOWER.encode(app_hash), "");

    node.skip_steps(100);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(data_encoding::HEXLOWER.encode(app_hash), "");
}
