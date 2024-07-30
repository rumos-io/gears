use gears::derive::{Protobuf, Query};
use serde::{Deserialize, Serialize};

mod inner {
    pub use gears::core::query::request::auth::QueryAccountRequest;
    pub use gears::core::query::response::auth::QueryAccountResponse;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryAccountsRequest;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryAccountsResponse;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryParamsRequest;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryParamsResponse;
}

#[derive(Clone, PartialEq, Message, Query)]
#[query(raw = "QueryParamsRequest", url = "/cosmos.auth.v1beta1.Query/Params")]
pub struct QueryParamsRequest {}

impl From<inner::QueryParamsRequest> for QueryParamsRequest {
    fn from(_value: inner::QueryParamsRequest) -> Self {
        QueryParamsRequest {}
    }
}

/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Protobuf, Query)]
#[proto(raw = "inner::QueryAccountResponse")]
pub struct QueryAccountResponse {
    /// account defines the account of the corresponding address.
    pub account: Option<Account>,
}

/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Debug, Protobuf, Query)]
#[proto(raw = "inner::QueryAccountRequest")]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    pub address: AccAddress,
}
