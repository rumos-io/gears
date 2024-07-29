use super::any::AnyTx;
use crate::{
    core::{errors::CoreError, Protobuf as CoreProtobuf},
    tendermint::{rpc::response::tx::Response, types::proto::event::Event},
    types::tx::{Tx, TxMessage},
};
use core_types::any::google::Any;
use prost::Message;
use serde::{Deserialize, Serialize};
use tendermint::types::proto::Protobuf;

/// TxResponse defines a structure containing relevant tx data and metadata. The
/// tags are stringified and the log is JSON decoded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TxResponse<M: TxMessage> {
    /// The block height
    pub height: i64,
    /// The transaction hash.
    pub tx_hash: String,
    /// Namespace for the Code
    pub codespace: String,
    /// Response code.
    pub code: u32,
    /// Result bytes, if any.
    pub data: String,
    /// The output of the application's logger (raw string). May be
    /// non-deterministic.
    pub raw_log: String,
    /// The output of the application's logger (typed). May be non-deterministic.
    pub logs: String, // TODO: may need to be typed in futureVec<AbciMessageLog>,
    /// Additional information. May be non-deterministic.
    pub info: String,
    /// Amount of gas requested for transaction.
    pub gas_wanted: i64,
    /// Amount of gas consumed by transaction.
    pub gas_used: i64,
    /// The request transaction bytes.
    pub tx: AnyTx<M>,
    /// Time of the previous block. For heights > 1, it's the weighted median of
    /// the timestamps of the valid votes in the block.LastCommit. For height == 1,
    /// it's genesis time.
    pub timestamp: String,
    /// Events defines all the events emitted by processing a transaction. Note,
    /// these events include those emitted by processing all the messages and those
    /// emitted from the ante. Whereas Logs contains the events, with
    /// additional metadata, emitted only by processing the messages.
    ///
    /// Since: cosmos-sdk 0.42.11, 0.44.5, 0.45
    pub events: Vec<Event>,
}

impl<M: TxMessage> TxResponse<M> {
    pub fn new_from_tx_response_and_string_time(
        tx_response: Response,
        timestamp: String,
    ) -> Result<Self, CoreError> {
        let cosmos_tx: Tx<M> = Tx::decode_vec(&tx_response.tx.clone())
            .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?;
        let any_tx = AnyTx::Tx(cosmos_tx);

        Ok(TxResponse {
            height: tx_response.height.into(),
            tx_hash: tx_response.hash.to_string(),
            codespace: tx_response.tx_result.codespace,
            code: tx_response.tx_result.code.value(),
            data: hex::encode(tx_response.tx_result.data),
            raw_log: tx_response.tx_result.log.clone(),
            logs: tx_response.tx_result.log,
            info: tx_response.tx_result.info,
            gas_wanted: tx_response.tx_result.gas_wanted,
            gas_used: tx_response.tx_result.gas_used,
            tx: any_tx,
            timestamp,
            events: tx_response
                .tx_result
                .events
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
}

impl<M: TxMessage> TryFrom<TxResponseRaw> for TxResponse<M> {
    type Error = CoreError;

    fn try_from(
        TxResponseRaw {
            height,
            tx_hash,
            codespace,
            code,
            data,
            raw_log,
            logs,
            info,
            gas_wanted,
            gas_used,
            tx,
            timestamp,
            events,
        }: TxResponseRaw,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            height,
            tx_hash,
            codespace,
            code,
            data,
            raw_log,
            logs,
            info,
            gas_wanted,
            gas_used,
            tx: tx
                .ok_or(CoreError::MissingField("Missed field 'tx'".to_string()))?
                .try_into()?,
            timestamp,
            events,
        })
    }
}

impl<M: TxMessage> CoreProtobuf<TxResponseRaw> for TxResponse<M> {}

#[derive(Clone, PartialEq, Deserialize, Serialize, Message)]
pub struct TxResponseRaw {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(string, tag = "2")]
    pub tx_hash: String,
    #[prost(string, tag = "3")]
    pub codespace: String,
    #[prost(uint32, tag = "4")]
    pub code: u32,
    #[prost(string, tag = "5")]
    pub data: String,
    #[prost(string, tag = "6")]
    pub raw_log: String,
    #[prost(string, tag = "7")]
    pub logs: String,
    #[prost(string, tag = "8")]
    pub info: String,
    #[prost(int64, tag = "9")]
    pub gas_wanted: i64,
    #[prost(int64, tag = "10")]
    pub gas_used: i64,
    #[prost(message, optional, tag = "11")]
    pub tx: Option<Any>,
    #[prost(string, tag = "12")]
    pub timestamp: String,
    #[prost(message, repeated, tag = "13")]
    pub events: Vec<Event>,
}

impl<M: TxMessage> From<TxResponse<M>> for TxResponseRaw {
    fn from(
        TxResponse {
            height,
            tx_hash,
            codespace,
            code,
            data,
            raw_log,
            logs,
            info,
            gas_wanted,
            gas_used,
            tx,
            timestamp,
            events,
        }: TxResponse<M>,
    ) -> Self {
        Self {
            height,
            tx_hash,
            codespace,
            code,
            data,
            raw_log,
            logs,
            info,
            gas_wanted,
            gas_used,
            tx: Some(tx.into()),
            timestamp,
            events,
        }
    }
}
