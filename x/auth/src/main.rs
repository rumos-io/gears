use gears::{derive::Protobuf, types::account::Account};

#[derive(Clone, PartialEq, Debug, Protobuf)]

// #[proto(raw = "inner::QueryAccountResponse")]
pub struct QueryAccountResponse {
    #[proto(kind = "message")]
    pub account: Option<Account>,
}

fn main() {}
