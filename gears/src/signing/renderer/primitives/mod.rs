//! Default implementation of formatting for primitive types as specified in Cosmos SDK ADR 50
//! https://docs.cosmos.network/main/build/architecture/adr-050-sign-mode-textual-annex1#message
//! NOTE: This implementation is not complete and should be extended with the remaining types
pub mod address;
pub mod bool;
pub mod bytes;
pub mod coin;
pub mod decimal256;
pub mod i64;
pub mod send_coins;
pub mod string;
pub mod u32;
pub mod u64;
pub mod uint256;
