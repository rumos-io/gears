use crate::rest::error::HTTPError;
use crate::types::pagination::response::PaginationResponse;
use crate::types::request::tx::BroadcastTxRequest;
use crate::types::response::any::AnyTx;
use crate::types::response::tx::{
    BroadcastTxResponse, BroadcastTxResponseLight, TxResponse, TxResponseLight,
};
use crate::types::response::tx_event::GetTxsEventResponse;
use crate::types::tx::{Tx, TxMessage};
use axum::extract::{Path, Query as AxumQuery, State};
use axum::Json;
use bytes::Bytes;
use core_types::Protobuf;
use ibc_proto::cosmos::tx::v1beta1::BroadcastMode;
use serde::{Deserialize, Serialize};
use tendermint::informal::node::Info;
use tendermint::informal::Hash;
use tendermint::rpc::client::{Client, HttpClient, HttpClientUrl};
use tendermint::rpc::query::Query;
use tendermint::rpc::response::tx::search::Response;
use tendermint::rpc::response::validators::Response as ValidatorsResponse;
use tendermint::rpc::url::Url;
use tendermint::rpc::Order;

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

pub async fn validatorsets_latest(
    AxumQuery(pagination): AxumQuery<Pagination>,
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<ValidatorsResponse>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let (page, limit) = parse_pagination(&pagination);
    let res = client
        .validators_latest(tendermint::rpc::client::Paging::Specific {
            page_number: (page as usize).into(),
            per_page: limit.into(),
        })
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            HTTPError::gateway_timeout()
        })?;
    Ok(Json(res))
}

pub async fn validatorsets(
    Path(height): Path<u32>,
    AxumQuery(pagination): AxumQuery<Pagination>,
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<ValidatorsResponse>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let (page, limit) = parse_pagination(&pagination);
    let res = client
        .validators(
            height,
            tendermint::rpc::client::Paging::Specific {
                page_number: (page as usize).into(),
                per_page: limit.into(),
            },
        )
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            HTTPError::gateway_timeout()
        })?;
    Ok(Json(res))
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
    let (page, limit) = parse_pagination(&pagination.0);

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

pub async fn tx<M: TxMessage>(
    Path(hash): Path<Hash>,
    State(tendermint_rpc_address): State<HttpClientUrl>,
) -> Result<Json<BroadcastTxResponse<M>>, HTTPError> {
    let client = HttpClient::new::<Url>(tendermint_rpc_address.into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");

    let res = client.tx(hash, true).await.ok();
    let res = if let Some(r) = res {
        Some(
            TxResponse::new_from_tx_response_and_string_time(r, "".to_string())
                .map_err(|_| HTTPError::internal_server_error())?,
        )
    } else {
        None
    };

    Ok(Json(BroadcastTxResponse {
        tx: res.as_ref().map(|r| r.tx.clone()).map(|tx| match tx {
            AnyTx::Tx(tx) => tx,
        }),
        tx_response: res,
    }))
}

pub async fn send_tx(
    state: State<HttpClientUrl>,
    tx_request: String,
) -> Result<Json<BroadcastTxResponseLight>, HTTPError> {
    let client = HttpClient::new::<Url>(state.0.clone().into()).expect("the conversion to Url then back to HttClientUrl should not be necessary, it will never fail, the dep needs to be fixed");
    let tx_request: BroadcastTxRequest =
        serde_json::from_str(&tx_request).map_err(|_| HTTPError::bad_gateway())?;

    let bytes = data_encoding::BASE64
        .decode(tx_request.tx_bytes.as_bytes())
        .map_err(|_| HTTPError::internal_server_error())?;

    let tx_response = if let Some(mode) = BroadcastMode::from_str_name(&tx_request.mode) {
        match mode {
            BroadcastMode::Sync => {
                let res = client
                    .broadcast_tx_sync(bytes)
                    .await
                    .map_err(|_| HTTPError::internal_server_error())?;
                TxResponseLight {
                    txhash: res.hash.to_string(),
                    code: res.code.into(),
                    raw_log: res.log,
                }
            }
            BroadcastMode::Async => {
                let res = client
                    .broadcast_tx_async(bytes)
                    .await
                    .map_err(|_| HTTPError::internal_server_error())?;
                TxResponseLight {
                    txhash: res.hash.to_string(),
                    code: res.code.into(),
                    raw_log: res.log,
                }
            }
            // TODO: is it a default value? keplr uses sync as default
            BroadcastMode::Block | BroadcastMode::Unspecified => {
                let res = client
                    .broadcast_tx_commit(bytes)
                    .await
                    .map_err(|_| HTTPError::internal_server_error())?;
                TxResponseLight {
                    txhash: res.hash.to_string(),
                    code: res.deliver_tx.code.into(),
                    raw_log: res.deliver_tx.log,
                }
            }
        }
    } else {
        return Err(HTTPError::internal_server_error());
    };

    Ok(Json(BroadcastTxResponseLight {
        tx_response: Some(tx_response),
    }))
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
