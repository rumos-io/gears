use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tendermint_informal::node::Info;
use tendermint_rpc::{Client, HttpClient};

use crate::client::rest::{error::Error, pagination::Pagination};
use crate::TM_ADDRESS;

// TODO:
// 1. handle multiple events in /cosmos/tx/v1beta1/txs request
// 2. include application information in NodeInfoResponse
// 3. get block in /cosmos/tx/v1beta1/txs so that the timestamp can be added to TxResponse

#[derive(Serialize, Deserialize)]
pub struct NodeInfoResponse {
    #[serde(rename = "default_node_info")]
    node_info: Info,
    //TODO: application_version
}

pub async fn node_info() -> Result<Json<NodeInfoResponse>, Error> {
    let client = HttpClient::new(TM_ADDRESS).expect("hard coded URL is valid");

    let res = client.status().await.map_err(|e| {
        tracing::error!("Error connecting to Tendermint: {e}");
        Error::gateway_timeout()
    })?;

    let node_info = NodeInfoResponse {
        node_info: res.node_info,
    };
    Ok(Json(node_info))
}
