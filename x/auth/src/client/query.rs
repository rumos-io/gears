use gears::{
    derive::{Protobuf, Query},
    types::{
        account::Account,
        address::AccAddress,
        pagination::{request::PaginationRequest, response::PaginationResponse},
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::AuthsParams;

mod inner {
    pub use gears::core::query::request::auth::QueryAccountRequest;
    pub use gears::core::query::response::auth::QueryAccountResponse;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryAccountsRequest;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryAccountsResponse;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryParamsRequest;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryParamsResponse;
}

#[derive(Clone, PartialEq, Message, Query, Protobuf)]
#[query(url = "/cosmos.auth.v1beta1.Query/Params")]
#[proto(raw = "inner::QueryParamsRequest")]
pub struct QueryParamsRequest {}

/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Protobuf, Query)]
#[proto(raw = "inner::QueryAccountResponse")]
pub struct QueryAccountResponse {
    /// account defines the account of the corresponding address.
    #[proto(optional)]
    pub account: Option<Account>,
}

/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Debug, Protobuf, Query)]
#[query(url = "/cosmos.auth.v1beta1.Query/Account")]
#[proto(raw = "inner::QueryAccountRequest")]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    pub address: AccAddress,
}

/// QueryAccountsRequest is the request type for the Query/Accounts RPC method.
#[derive(Clone, PartialEq, Debug, Query, Protobuf)]
#[query(url = "/cosmos.auth.v1beta1.Query/Accounts")]
#[proto(raw = "inner::QueryAccountsRequest")]
pub struct QueryAccountsRequest {
    /// pagination defines an optional pagination for the request.
    #[proto(optional)]
    pub pagination: Option<PaginationRequest>,
}

/// QueryAccountsResponse is the response type for the Query/Accounts RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query, Protobuf)]
#[proto(raw = "inner::QueryAccountsResponse")]
pub struct QueryAccountsResponse {
    /// accounts are the existing accounts
    #[proto(repeated)]
    pub accounts: Vec<Account>,
    /// pagination defines the pagination in the response.
    #[proto(optional)]
    pub pagination: Option<PaginationResponse>,
}

/// QueryParamsResponse is the response type for the Query/Params RPC method
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Query, Protobuf)]
#[proto(raw = "inner::QueryParamsResponse")]
pub struct QueryParamsResponse {
    #[proto(optional)]
    pub params: AuthsParams,
}
