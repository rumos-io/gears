use std::str::FromStr;

use gears::{
    extensions::testing::UnwrapTesting,
    tendermint::types::time::timestamp::Timestamp,
    types::{base::coin::UnsignedCoin, decimal256::Decimal256, uint::Uint256},
};
use utils::{set_node, MockBankKeeper, MockStakingKeeper};

#[path = "./utils.rs"]
mod utils;

#[test]
fn test_init_and_few_blocks() {
    let mut node = set_node(None, None);

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

#[test]
fn test_init_and_few_blocks_with_tokens() {
    let mut node = set_node(
        Some(MockBankKeeper::new(
            UnsignedCoin::from_str("1000000000000uatom").unwrap_test(),
            None,
        )),
        Some(MockStakingKeeper::new(Decimal256::new(Uint256::from(
            1000000000_u64,
        )))),
    );

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;

    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "490212363fbd9a59250c6a8e329a6b6f39ebf95c50535f8d4ddf461577a34902"
    );

    node.skip_steps(100);

    let app_hash = &node.step(vec![], Timestamp::UNIX_EPOCH).app_hash;
    assert_eq!(
        data_encoding::HEXLOWER.encode(app_hash),
        "15bd8b0fd25f63bb734e67bb355c189c75836a9331066975de8c095822529c09"
    );
}
