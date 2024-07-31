use crate::{
    baseapp::Query,
    core::errors::CoreError,
    error::IBC_ENCODE_UNWRAP,
    tendermint::informal::Hash,
    types::{
        response::{
            tx::{TxResponse, TxResponseRaw},
            tx_event::{SearchTxsResult, SearchTxsResultRaw},
        },
        tx::TxMessage,
    },
};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tendermint::types::proto::Protobuf;

mod inner {
    pub use ibc_proto::cosmos::tx::v1beta1::GetTxRequest;
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
    #[prost(message, optional, tag = "1")]
    pub txs: Option<SearchTxsResultRaw>,
}

impl<M: TxMessage> From<QueryTxsResponse<M>> for QueryTxsResponseRaw {
    fn from(QueryTxsResponse { txs }: QueryTxsResponse<M>) -> Self {
        Self {
            txs: Some(txs.into()),
        }
    }
}

/// QueryTxsResponse is the response type for tendermint rpc query
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct QueryTxsResponse<M: TxMessage> {
    pub txs: SearchTxsResult<M>,
}

impl<M: TxMessage> TryFrom<QueryTxsResponseRaw> for QueryTxsResponse<M> {
    type Error = CoreError;

    fn try_from(QueryTxsResponseRaw { txs }: QueryTxsResponseRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            txs: txs
                .ok_or(CoreError::MissingField("Field 'txs' is missed".to_string()))?
                .try_into()?,
        })
    }
}

impl<M: TxMessage> Protobuf<QueryTxsResponseRaw> for QueryTxsResponse<M> {}

#[derive(Clone, PartialEq)]
pub struct QueryGetTxRequest {
    pub hash: Hash,
}

impl Query for QueryGetTxRequest {
    fn query_url(&self) -> &'static str {
        "/cosmos.auth.v1beta1.Query/QueryGetTx"
    }

    fn into_bytes(self) -> Vec<u8> {
        self.encode_vec().expect(IBC_ENCODE_UNWRAP)
    }
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

impl Protobuf<inner::GetTxRequest> for QueryGetTxRequest {}

#[derive(Clone, PartialEq, Message)]
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

impl Query for QueryGetTxsEventRequest {
    fn query_url(&self) -> &'static str {
        "/cosmos.auth.v1beta1.Query/QueryGetTxsEvent"
    }

    fn into_bytes(self) -> Vec<u8> {
        self.encode_vec().expect(IBC_ENCODE_UNWRAP)
    }
}

impl Protobuf<QueryGetTxsEventRequest> for QueryGetTxsEventRequest {}
