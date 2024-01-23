mod accessor;
pub mod persistent;
pub mod tree;
#[allow(dead_code)]
pub mod tree_v3;
pub use tree::*;

pub const HASH_LENGHT: usize = 32;
