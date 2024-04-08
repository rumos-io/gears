use ibc_proto::protobuf::Protobuf;
use serde::{Deserialize, Serialize};

use crate::{account::Account, any::google::Any, errors::Error};

pub mod inner {
    pub use ibc_proto::cosmos::auth::v1beta1::QueryAccountResponse;
}

/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct QueryAccountResponse {
    /// account defines the account of the corresponding address.
    pub account: Account,
}

impl TryFrom<inner::QueryAccountResponse> for QueryAccountResponse {
    type Error = Error;

    fn try_from(raw: inner::QueryAccountResponse) -> Result<Self, Self::Error> {
        let account = raw
            .account
            .map(Any::from)
            .ok_or(Error::MissingField("account".into()))?
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
