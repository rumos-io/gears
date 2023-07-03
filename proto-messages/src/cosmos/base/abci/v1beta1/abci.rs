use serde::{Deserialize, Serialize};
use tendermint_informal::abci::Event;

use crate::cosmos::tx::v1beta1::{AnyTx, Message};

/// TxResponse defines a structure containing relevant tx data and metadata. The
/// tags are stringified and the log is JSON decoded.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TxResponse<M: Message> {
    /// The block height
    pub height: i64,
    /// The transaction hash.
    pub txhash: String,
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

// pub struct DeliverTx {
//     /// The response code.
//     ///
//     /// This code should be `0` only if the transaction is fully valid. However,
//     /// invalid transactions included in a block will still be executed against
//     /// the application state.
//     pub code: Code,
//     /// Result bytes, if any.
//     #[serde(with = "serializers::nullable")]
//     pub data: Bytes,
//     /// The output of the application's logger.
//     ///
//     /// **May be non-deterministic**.
//     pub log: String,
//     /// Additional information.
//     ///
//     /// **May be non-deterministic**.
//     pub info: String,
//     /// Amount of gas requested for the transaction.
//     #[serde(with = "serializers::from_str")]
//     pub gas_wanted: i64,
//     /// Amount of gas consumed by the transaction.
//     #[serde(with = "serializers::from_str")]
//     pub gas_used: i64,
//     /// Events that occurred while executing the transaction.
//     pub events: Vec<Event>,
//     /// The namespace for the `code`.
//     pub codespace: String,
// }
