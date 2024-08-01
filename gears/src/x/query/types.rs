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
use protobuf_derive::Raw;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tendermint::types::proto::Protobuf;

mod inner {
    pub use ibc_proto::cosmos::tx::v1beta1::GetTxRequest;
}

/// QueryTxResponse is the response type for tendermint rpc query
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Raw)]
pub struct QueryTxResponse<M: TxMessage> {
    #[raw(kind(message), raw = TxResponseRaw, optional)]
    pub tx: TxResponse<M>,
}

impl<M: TxMessage> From<QueryTxResponse<M>> for RawQueryTxResponse {
    fn from(QueryTxResponse { tx }: QueryTxResponse<M>) -> Self {
        Self {
            tx: Some(tx.into()),
        }
    }
}

impl<M: TxMessage> TryFrom<RawQueryTxResponse> for QueryTxResponse<M> {
    type Error = CoreError;

    fn try_from(RawQueryTxResponse { tx }: RawQueryTxResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            tx: tx
                .ok_or(CoreError::MissingField("Missed field 'tx'".to_string()))?
                .try_into()
                .map_err(|e| CoreError::DecodeGeneral(format!("{e}")))?,
        })
    }
}

impl<M: TxMessage> Protobuf<RawQueryTxResponse> for QueryTxResponse<M> {}

/// QueryTxsResponse is the response type for tendermint rpc query
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Raw)]
pub struct QueryTxsResponse<M: TxMessage> {
    #[raw(kind(message), raw = SearchTxsResultRaw, optional)]
    pub txs: SearchTxsResult<M>,
}

impl<M: TxMessage> From<QueryTxsResponse<M>> for RawQueryTxsResponse {
    fn from(QueryTxsResponse { txs }: QueryTxsResponse<M>) -> Self {
        Self {
            txs: Some(txs.into()),
        }
    }
}

impl<M: TxMessage> TryFrom<RawQueryTxsResponse> for QueryTxsResponse<M> {
    type Error = CoreError;

    fn try_from(RawQueryTxsResponse { txs }: RawQueryTxsResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            txs: txs
                .ok_or(CoreError::MissingField("Field 'txs' is missed".to_string()))?
                .try_into()?,
        })
    }
}

impl<M: TxMessage> Protobuf<RawQueryTxsResponse> for QueryTxsResponse<M> {}

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
