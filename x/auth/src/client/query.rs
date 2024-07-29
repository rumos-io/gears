use crate::AuthsParams;
use gears::{
    core::{errors::CoreError, Protobuf},
    derive::Query,
    tendermint::informal::Hash,
    types::{
        account::Account,
        address::AccAddress,
        pagination::{request::PaginationRequest, response::PaginationResponse},
        response::tx::{TxResponse, TxResponseRaw},
        tx::TxMessage,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

mod inner {
    pub use gears::core::query::request::auth::QueryAccountRequest;
    pub use gears::core::query::response::auth::QueryAccountResponse;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryAccountsRequest;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryAccountsResponse;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryParamsRequest;
    pub use ibc_proto::cosmos::auth::v1beta1::QueryParamsResponse;
    pub use ibc_proto::cosmos::tx::v1beta1::GetTxRequest;
}

/// QueryAccountResponse is the response type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[query(kind = "response", raw = "inner::QueryAccountResponse")]
pub struct QueryAccountResponse {
    /// account defines the account of the corresponding address.
    pub account: Option<Account>,
}

impl TryFrom<inner::QueryAccountResponse> for QueryAccountResponse {
    type Error = CoreError;

    fn try_from(raw: inner::QueryAccountResponse) -> Result<Self, Self::Error> {
        let account = raw.account.map(|a| a.try_into()).transpose()?;
        Ok(QueryAccountResponse { account })
    }
}

impl From<QueryAccountResponse> for inner::QueryAccountResponse {
    fn from(query: QueryAccountResponse) -> inner::QueryAccountResponse {
        Self {
            account: query.account.map(Into::into),
        }
    }
}

/// QueryAccountsResponse is the response type for the Query/Accounts RPC method.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Query)]
#[query(kind = "response", raw = "inner::QueryAccountsResponse")]
pub struct QueryAccountsResponse {
    /// accounts are the existing accounts
    pub accounts: Vec<Account>,
    /// pagination defines the pagination in the response.
    pub pagination: Option<PaginationResponse>,
}

impl TryFrom<inner::QueryAccountsResponse> for QueryAccountsResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryAccountsResponse {
            accounts,
            pagination,
        }: inner::QueryAccountsResponse,
    ) -> Result<Self, Self::Error> {
        let mut accounts_res = Vec::with_capacity(accounts.len());
        for raw in accounts {
            accounts_res.push(raw.try_into()?);
        }
        Ok(QueryAccountsResponse {
            accounts: accounts_res,
            pagination: pagination.map(Into::into),
        })
    }
}

impl From<QueryAccountsResponse> for inner::QueryAccountsResponse {
    fn from(
        QueryAccountsResponse {
            accounts,
            pagination,
        }: QueryAccountsResponse,
    ) -> inner::QueryAccountsResponse {
        Self {
            accounts: accounts.into_iter().map(Into::into).collect(),
            pagination: pagination.map(Into::into),
        }
    }
}

/// QueryParamsResponse is the response type for the Query/Params RPC method
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Query)]
#[query(raw = "inner::QueryParamsResponse")]
pub struct QueryParamsResponse {
    pub params: AuthsParams,
}

impl From<QueryParamsResponse> for inner::QueryParamsResponse {
    fn from(QueryParamsResponse { params }: QueryParamsResponse) -> Self {
        Self {
            params: Some(params.into()),
        }
    }
}

impl TryFrom<inner::QueryParamsResponse> for QueryParamsResponse {
    type Error = CoreError;

    fn try_from(
        inner::QueryParamsResponse { params }: inner::QueryParamsResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            params: params
                .ok_or(CoreError::MissingField("Missing field 'params'.".into()))?
                .into(),
        })
    }
}

#[derive(Clone, Message)]
pub struct QueryTxResponseRaw {
    #[prost(message, optional)]
    pub tx: Option<TxResponseRaw>,
}

impl<M: TxMessage> From<QueryTxResponse<M>> for QueryTxResponseRaw {
    fn from(QueryTxResponse { tx }: QueryTxResponse<M>) -> Self {
        Self {
            tx: Some(tx.into()),
        }
    }
}

/// QueryTxResponse is the response type for tendermint rpc query
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct QueryTxResponse<M: TxMessage> {
    pub tx: TxResponse<M>,
}

impl<M: TxMessage> TryFrom<QueryTxResponseRaw> for QueryTxResponse<M> {
    type Error = CoreError;

    fn try_from(QueryTxResponseRaw { tx }: QueryTxResponseRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            tx: tx
                .ok_or(CoreError::MissingField("Missed field 'tx'".to_string()))?
                .try_into()
                .map_err(|e| CoreError::DecodeGeneral(format!("{e}")))?,
        })
    }
}

impl<M: TxMessage> Protobuf<QueryTxResponseRaw> for QueryTxResponse<M> {}

#[derive(Clone, Message)]
pub struct QueryTxsResponseRaw {
    #[prost(message, repeated)]
    pub txs: Vec<TxResponseRaw>,
}

impl<M: TxMessage> From<QueryTxsResponse<M>> for QueryTxsResponseRaw {
    fn from(QueryTxsResponse { txs }: QueryTxsResponse<M>) -> Self {
        Self {
            txs: txs.into_iter().map(Into::into).collect(),
        }
    }
}

/// QueryTxsResponse is the response type for tendermint rpc query
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct QueryTxsResponse<M: TxMessage> {
    pub txs: Vec<TxResponse<M>>,
}

impl<M: TxMessage> TryFrom<QueryTxsResponseRaw> for QueryTxsResponse<M> {
    type Error = CoreError;

    fn try_from(QueryTxsResponseRaw { txs }: QueryTxsResponseRaw) -> Result<Self, Self::Error> {
        let mut txs_res = vec![];
        for tx in txs {
            txs_res.push(
                tx.try_into()
                    .map_err(|e| CoreError::DecodeGeneral(format!("{e}")))?,
            )
        }
        Ok(Self { txs: txs_res })
    }
}

impl<M: TxMessage> Protobuf<QueryTxsResponseRaw> for QueryTxsResponse<M> {}

/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    kind = "request",
    raw = "inner::QueryAccountRequest",
    url = "/cosmos.auth.v1beta1.Query/Account"
)]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    pub address: AccAddress,
}

impl TryFrom<inner::QueryAccountRequest> for QueryAccountRequest {
    type Error = CoreError;

    fn try_from(raw: inner::QueryAccountRequest) -> Result<Self, Self::Error> {
        let address = AccAddress::from_bech32(&raw.address)
            .map_err(|e| Self::Error::DecodeAddress(e.to_string()))?;

        Ok(QueryAccountRequest { address })
    }
}

impl From<QueryAccountRequest> for inner::QueryAccountRequest {
    fn from(query: QueryAccountRequest) -> inner::QueryAccountRequest {
        Self {
            address: query.address.to_string(),
        }
    }
}

/// QueryAccountsRequest is the request type for the Query/Accounts RPC method.
#[derive(Clone, PartialEq, Debug, Query)]
#[query(
    kind = "request",
    raw = "inner::QueryAccountsRequest",
    url = "/cosmos.auth.v1beta1.Query/Accounts"
)]
pub struct QueryAccountsRequest {
    /// pagination defines an optional pagination for the request.
    pub pagination: PaginationRequest,
}

impl TryFrom<inner::QueryAccountsRequest> for QueryAccountsRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryAccountsRequest { pagination }: inner::QueryAccountsRequest,
    ) -> Result<Self, Self::Error> {
        Ok(QueryAccountsRequest {
            pagination: pagination
                .ok_or(CoreError::MissingField(
                    "Missing field 'pagination'.".into(),
                ))?
                .into(),
        })
    }
}

impl From<QueryAccountsRequest> for inner::QueryAccountsRequest {
    fn from(
        QueryAccountsRequest { pagination }: QueryAccountsRequest,
    ) -> inner::QueryAccountsRequest {
        Self {
            pagination: Some(pagination.into()),
        }
    }
}

#[derive(Clone, PartialEq, Message, Query)]
#[query(raw = "QueryParamsRequest", url = "/cosmos.auth.v1beta1.Query/Params")]
pub struct QueryParamsRequest {}

impl From<inner::QueryParamsRequest> for QueryParamsRequest {
    fn from(_value: inner::QueryParamsRequest) -> Self {
        QueryParamsRequest {}
    }
}

#[derive(Clone, PartialEq, Query)]
#[query(
    raw = "inner::GetTxRequest",
    url = "/cosmos.auth.v1beta1.Query/QueryGetTx"
)]
pub struct QueryGetTxRequest {
    pub hash: Hash,
}

impl From<QueryGetTxRequest> for inner::GetTxRequest {
    fn from(QueryGetTxRequest { hash }: QueryGetTxRequest) -> Self {
        inner::GetTxRequest {
            hash: hash.to_string(),
        }
    }
}

impl TryFrom<inner::GetTxRequest> for QueryGetTxRequest {
    type Error = CoreError;

    fn try_from(inner::GetTxRequest { hash }: inner::GetTxRequest) -> Result<Self, Self::Error> {
        Ok(QueryGetTxRequest {
            hash: Hash::from_str(&hash).map_err(|e| CoreError::DecodeGeneral(format!("{e}")))?,
        })
    }
}

#[derive(Clone, PartialEq, Message, Query)]
#[query(
    raw = "QueryGetTxsEventRequest",
    url = "/cosmos.auth.v1beta1.Query/QueryGetTxsEvent"
)]
pub struct QueryGetTxsEventRequest {
    #[prost(string, repeated, tag = "1")]
    pub events: Vec<String>,
    #[prost(string, tag = "2")]
    pub order_by: String,
    #[prost(uint32, tag = "4")]
    pub page: u32,
    #[prost(uint32, tag = "5")]
    pub limit: u32,
}
