use rocket::{get, serde::json::Json};
use serde::{Deserialize, Serialize};
use tendermint_informal::node::Info;
use tendermint_rpc::{endpoint::tx_search::Response, query::Query, Order};
use tendermint_rpc::{Client, HttpClient};

use super::pagination::parse_pagination;
use crate::{
    client::rest::{error::Error, pagination::Pagination},
    TM_ADDRESS,
};

// TODO:
// 1. handle multiple events in /cosmos/tx/v1beta1/txs endpoint
// 2. include application information in NodeInfoResponse

#[derive(Serialize, Deserialize)]
pub struct NodeInfoResponse {
    #[serde(rename = "default_node_info")]
    node_info: Info,
    //TODO: application_version
}

#[get("/cosmos/base/tendermint/v1beta1/node_info")]
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

#[get("/cosmos/tx/v1beta1/txs?<events>&<pagination>")]
pub async fn txs(events: &str, pagination: Pagination) -> Result<Json<Response>, Error> {
    let client = HttpClient::new(TM_ADDRESS).expect("hard coded URL is valid");

    let query: Query = events
        .parse()
        .map_err(|e: tendermint_rpc::error::Error| Error::bad_request(e.detail().to_string()))?;
    let (page, limit) = parse_pagination(pagination);

    let res = client
        .tx_search(query, false, page, limit, Order::Descending)
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            Error::gateway_timeout()
        })?;

    Ok(Json(res))
}
