use ibc_proto::any::google::Any;
use serde::{Deserialize, Serialize};
use tendermint::types::proto::Protobuf;

use crate::types::account::Account;

mod inner {
    pub use ibc_proto::query::response::account::QueryAccountResponse;
}

/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryAccountResponse {
    /// account defines the account of the corresponding address.
    pub account: Account,
}

impl TryFrom<inner::QueryAccountResponse> for QueryAccountResponse {
    type Error = ibc_proto::errors::Error;

    fn try_from(raw: inner::QueryAccountResponse) -> Result<Self, Self::Error> {
        let account = raw
            .account
            .map(Any::from)
            .ok_or(ibc_proto::errors::Error::MissingField("account".into()))?
            .try_into()?;

        Ok(QueryAccountResponse { account })
    }
}

impl From<QueryAccountResponse> for inner::QueryAccountResponse {
    fn from(query: QueryAccountResponse) -> inner::QueryAccountResponse {
        Self {
            account: Some(Any::from(query.account).into()),
        }
    }
}

impl Protobuf<inner::QueryAccountResponse> for QueryAccountResponse {}
