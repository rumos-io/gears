use gears::tendermint::types::time::timestamp::Timestamp;
use utils::set_node;

#[path = "./utils.rs"]
mod utils;

#[test]
fn test_init_and_few_blocks() {
    let mut node = set_node();

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;

    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "dca0afec3e333f4eb5d48a074ec6a861dbf945f6cc04e8e880e76b4136857180"
    );

    node.skip_steps(100);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "548e264ab9174c4c366abd27c0d0c888fa591977145d314f9c548e60160f01d4"
    );
}
