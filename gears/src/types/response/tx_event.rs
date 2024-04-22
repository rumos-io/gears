use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::types::tx::Tx;
use crate::types::tx::TxMessage;

use super::tx::TxResponse;
use super::PageResponse;

/// GetTxsEventResponse is the response type for the Service.TxsByEvents
/// RPC method.
#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTxsEventResponse<M: TxMessage> {
    /// txs is the list of queried transactions.
    pub txs: Vec<Tx<M>>,
    /// tx_responses is the list of queried TxResponses.
    pub tx_responses: Vec<TxResponse<M>>,
    /// pagination defines a pagination for the response.
    /// Deprecated post v0.46.x: use total instead.
    // TODO: doesn't serialize correctly - has been deprecated
    pub pagination: Option<PageResponse>,
    /// total is total number of results available
    #[serde_as(as = "DisplayFromStr")]
    pub total: u64,
}
