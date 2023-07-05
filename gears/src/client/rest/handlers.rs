use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::Json;
use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::bank::v1beta1::QueryTotalSupplyResponse;
use proto_messages::cosmos::tx::v1beta1::Message;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use store_crate::StoreKey;
use strum::IntoEnumIterator;
use tendermint_abci::Application;
use tendermint_informal::node::Info;
use tendermint_proto::abci::RequestQuery;
use tendermint_rpc::{Client, HttpClient};

use crate::baseapp::ante::{AuthKeeper, BankKeeper};
use crate::baseapp::{BaseApp, Handler};
use crate::client::rest::{error::Error, pagination::Pagination};
use crate::types::context::QueryContext;
use crate::x::params::ParamsSubspaceKey;
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

// #[get("/cosmos/tx/v1beta1/txs?<events>&<pagination>")]
// pub async fn txs<M: Message>(
//     events: &str,
//     pagination: Pagination,
// ) -> Result<Json<GetTxsEventResponse<M>>, Error> {
//     let client = HttpClient::new(TM_ADDRESS).expect("hard coded URL is valid");

//     let query: Query = events
//         .parse()
//         .map_err(|e: tendermint_rpc::error::Error| Error::bad_request(e.detail().to_string()))?;
//     let (page, limit) = parse_pagination(pagination);

//     let res_tx = client
//         .tx_search(query, false, page, limit, Order::Descending)
//         .await
//         .map_err(|e| {
//             tracing::error!("Error connecting to Tendermint: {e}");
//             Error::gateway_timeout()
//         })?;

//     let res = map_responses(res_tx)?;

//     Ok(Json(res))
// }

// /// Maps a tendermint tx_search response to a Cosmos get txs by event response
// fn map_responses<M: Message>(res_tx: Response) -> Result<GetTxsEventResponse<M>, Error> {
//     let mut tx_responses = Vec::with_capacity(res_tx.txs.len());
//     let mut txs = Vec::with_capacity(res_tx.txs.len());

//     for tx in res_tx.txs {
//         let cosmos_tx = Tx::decode::<Bytes>(tx.tx.into()).map_err(|_| Error::bad_gateway())?;
//         txs.push(cosmos_tx.clone());

//         let any_tx = AnyTx::Tx(cosmos_tx);

//         tx_responses.push(TxResponse {
//             height: tx.height.into(),
//             txhash: tx.hash.to_string(),
//             codespace: tx.tx_result.codespace,
//             code: tx.tx_result.code.value(),
//             data: hex::encode(tx.tx_result.data),
//             raw_log: tx.tx_result.log.clone(),
//             logs: tx.tx_result.log,
//             info: tx.tx_result.info,
//             gas_wanted: tx.tx_result.gas_wanted,
//             gas_used: tx.tx_result.gas_used,
//             tx: any_tx,
//             timestamp: "".into(), // TODO: need to get the blocks for this
//             events: tx.tx_result.events.into_iter().map(|e| e.into()).collect(),
//         });
//     }

//     let total = txs.len().try_into().map_err(|_| Error::bad_gateway())?;

//     Ok(GetTxsEventResponse {
//         pagination: Some(PageResponse {
//             next_key: vec![],
//             total,
//         }),
//         total,
//         txs,
//         tx_responses,
//     })
// }

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
