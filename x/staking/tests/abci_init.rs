use gears::tendermint::types::time::timestamp::Timestamp;

use utils::set_node;

#[path = "./utils.rs"]
mod utils;

#[test]
/// In this scenario, we test the initialization of the application and execute a few blocks
fn test_init_and_few_blocks() {
    let mut node = set_node();

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "f65fe214029e11072006ffd1ddbadc37f079533c46169a4a469bedea28cf20ce"
    );

    node.skip_steps(100);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "03ff97724d841c6427306b7e1d726768d45bfac0b96e52b55e57159656daff16"
    );
}
