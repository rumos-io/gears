use serde::{Deserialize, Serialize};

use crate::cosmos::{
    base::abci::v1beta1::TxResponse,
    tx::v1beta1::{message::Message, tx::tx::Tx},
};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

/// GetTxsEventResponse is the response type for the Service.TxsByEvents
/// RPC method.
#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTxsEventResponse<M: Message> {
    /// txs is the list of queried transactions.
    pub txs: Vec<Tx<M>>,
    /// tx_responses is the list of queried TxResponses.
    pub tx_responses: Vec<TxResponse<M>>,
    /// pagination defines a pagination for the response.
    /// Deprecated post v0.46.x: use total instead.
    // TODO: doesn't serialize correctly - has been deprecated
    pub pagination: Option<ibc_proto::cosmos::base::query::v1beta1::PageResponse>,
    /// total is total number of results available
    #[serde_as(as = "DisplayFromStr")]
    pub total: u64,
}
