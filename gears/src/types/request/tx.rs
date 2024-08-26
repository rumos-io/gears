use serde::{Deserialize, Serialize};

/// BroadcastTxRequest is the request type for the Service.BroadcastTxRequest
/// RPC method.
// the ibc-proto type has another representation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadcastTxRequest {
    pub tx_bytes: String,
    pub mode: String,
}
