use gears::derive::{Protobuf, Query};
use serde::{Deserialize, Serialize};

use gears::types::{account::Account, address::AccAddress};

mod inner {
    pub use gears::core::query::request::account::QueryAccountRequest;
    pub use gears::core::query::response::account::QueryAccountResponse;
}

/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Protobuf, Query)]
#[query(kind = "response")]
#[proto(raw = "inner::QueryAccountResponse")]
pub struct QueryAccountResponse {
    /// account defines the account of the corresponding address.
    pub account: Option<Account>,
}

/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Debug, Protobuf, Query)]
#[query(kind = "request", url = "/cosmos.auth.v1beta1.Query/Account")]
#[proto(raw = "inner::QueryAccountRequest")]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    pub address: AccAddress,
}
