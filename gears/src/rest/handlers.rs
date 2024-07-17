use crate::rest::error::HTTPError;
use crate::types::pagination::response::PaginationResponse;
use crate::types::response::any::AnyTx;
use crate::types::response::tx::TxResponse;
use crate::types::response::tx_event::GetTxsEventResponse;
use crate::types::tx::{Tx, TxMessage};
use axum::extract::{Query as AxumQuery, State};
use axum::Json;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tendermint::informal::node::Info;
use tendermint::rpc::client::{Client, HttpClient, HttpClientUrl};
use tendermint::rpc::query::Query;
use tendermint::rpc::response::tx::search::Response;
use tendermint::rpc::url::Url;
use tendermint::rpc::Order;
use tendermint::types::proto::Protobuf;

use super::{parse_pagination, Pagination};

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

pub async fn node_info(
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<NodeInfoResponse>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let res = client.status().await.map_err(|e| {
        tracing::error!("Error connecting to Tendermint: {e}");
        HTTPError::gateway_timeout()
    })?;

    let node_info = NodeInfoResponse {
        node_info: res.node_info,
    };
    Ok(Json(node_info))
}

#[derive(Deserialize)]
pub struct RawEvents {
    events: String,
}

pub async fn txs<M: TxMessage>(
    events: AxumQuery<RawEvents>,
    pagination: AxumQuery<Pagination>,
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<GetTxsEventResponse<M>>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let query: Query = events
        .0
        .events
        .parse()
        .map_err(|e: tendermint::rpc::error::Error| {
            HTTPError::bad_request(e.detail().to_string())
        })?;
    let (page, limit) = parse_pagination(pagination.0.clone());

    let res_tx = client
        .tx_search(query, false, page, limit, Order::Descending)
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            HTTPError::gateway_timeout()
        })?;

    let res = map_responses(res_tx)?;

    Ok(Json(res))
}

// Maps a tendermint tx_search response to a Cosmos get txs by event response
fn map_responses<M: TxMessage>(res_tx: Response) -> Result<GetTxsEventResponse<M>, HTTPError> {
    let mut tx_responses = Vec::with_capacity(res_tx.txs.len());
    let mut txs = Vec::with_capacity(res_tx.txs.len());

    for tx in res_tx.txs {
        let cosmos_tx = Tx::decode::<Bytes>(tx.tx.into()).map_err(|_| HTTPError::bad_gateway())?;
        txs.push(cosmos_tx.clone());

        let any_tx = AnyTx::Tx(cosmos_tx);

        tx_responses.push(TxResponse {
            height: tx.height.into(),
            txhash: tx.hash.to_string(),
            codespace: tx.tx_result.codespace,
            code: tx.tx_result.code.value(),
            data: hex::encode(tx.tx_result.data),
            raw_log: tx.tx_result.log.clone(),
            logs: tx.tx_result.log,
            info: tx.tx_result.info,
            gas_wanted: tx.tx_result.gas_wanted,
            gas_used: tx.tx_result.gas_used,
            tx: any_tx,
            timestamp: "".into(), // TODO: need to get the blocks for this
            events: tx.tx_result.events.into_iter().map(Into::into).collect(),
        });
    }

    let total = txs.len().try_into().map_err(|_| HTTPError::bad_gateway())?;

    Ok(GetTxsEventResponse {
        pagination: Some(PaginationResponse {
            next_key: vec![],
            total,
        }),
        total,
        txs,
        tx_responses,
    })
}

// This is a hack for now to make the front end work
// TODO: remove this once the staking module is implemented
//#[get("/cosmos/staking/v1beta1/params")]
pub async fn staking_params() -> &'static str {
    r#"
    {
        "params": {
          "unbonding_time": "0",
          "max_validators": 0,
          "max_entries": 0,
          "historical_entries": 0,
          "bond_denom": "uatom",
          "min_commission_rate": "0"
        }
      }
    "#
}

pub async fn block_latest(
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<tendermint::rpc::endpoint::Response>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let res = client.latest_block().await.map_err(|e| {
        tracing::error!("Error connecting to Tendermint: {e}");
        HTTPError::gateway_timeout()
    })?;
    Ok(Json(res))
}
