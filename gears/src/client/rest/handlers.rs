use bytes::Bytes;
use ibc_proto::cosmos::base::query::v1beta1::PageResponse;

use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::bank::v1beta1::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest, QueryBalanceResponse,
    QueryTotalSupplyResponse,
};

use proto_messages::cosmos::base::abci::v1beta1::TxResponse;
use proto_messages::cosmos::tx::v1beta1::{AnyTx, GetTxsEventResponse, Tx};
use proto_types::AccAddress;
use rocket::State;
use rocket::{get, serde::json::Json};
use serde::{Deserialize, Serialize};
use tendermint_informal::node::Info;
use tendermint_rpc::{endpoint::tx_search::Response, query::Query, Order};
use tendermint_rpc::{Client, HttpClient};

use super::pagination::parse_pagination;
use crate::app::BaseApp;
use crate::types::QueryContext;
use crate::x::bank;
use crate::{
    client::rest::{error::Error, pagination::Pagination},
    TM_ADDRESS,
};

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
pub async fn txs(events: &str, pagination: Pagination) -> Result<Json<GetTxsEventResponse>, Error> {
    let client = HttpClient::new(TM_ADDRESS).expect("hard coded URL is valid");

    let query: Query = events
        .parse()
        .map_err(|e: tendermint_rpc::error::Error| Error::bad_request(e.detail().to_string()))?;
    let (page, limit) = parse_pagination(pagination);

    let res_tx = client
        .tx_search(query, false, page, limit, Order::Descending)
        .await
        .map_err(|e| {
            tracing::error!("Error connecting to Tendermint: {e}");
            Error::gateway_timeout()
        })?;

    let res = map_responses(res_tx)?;

    Ok(Json(res))
}

/// Maps a tendermint tx_search response to a Cosmos get txs by event response
fn map_responses(res_tx: Response) -> Result<GetTxsEventResponse, Error> {
    let mut tx_responses = Vec::with_capacity(res_tx.txs.len());
    let mut txs = Vec::with_capacity(res_tx.txs.len());

    for tx in res_tx.txs {
        let cosmos_tx = Tx::decode::<Bytes>(tx.tx.into()).map_err(|_| Error::bad_gateway())?;
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
            events: tx.tx_result.events.into_iter().map(|e| e.into()).collect(),
        });
    }

    let total = txs.len().try_into().map_err(|_| Error::bad_gateway())?;

    Ok(GetTxsEventResponse {
        pagination: Some(PageResponse {
            next_key: vec![],
            total,
        }),
        total,
        txs,
        tx_responses,
    })
}

/// Gets the total supply of every denom
#[get("/cosmos/bank/v1beta1/supply")]
pub async fn supply(app: &State<BaseApp>) -> Json<QueryTotalSupplyResponse> {
    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    let coins = bank::Bank::get_paginated_total_supply(&ctx);

    Json(QueryTotalSupplyResponse {
        supply: coins,
        pagination: None,
    })
}

/// Get all balances for a given address
#[get("/cosmos/bank/v1beta1/balances/<addr>?<pagination>")]
#[allow(unused_variables)]
pub async fn get_balances(
    app: &State<BaseApp>,
    addr: String,
    pagination: Pagination,
) -> Result<Json<QueryAllBalancesResponse>, Error> {
    let req = QueryAllBalancesRequest {
        address: AccAddress::from_bech32(&addr).map_err(|e| Error::bad_request(e.to_string()))?,
        pagination: None,
    };

    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(bank::Bank::query_all_balances(&ctx, req)))
}

/// Get balance for a given address and denom
#[get("/cosmos/bank/v1beta1/balances/<addr>/by_denom?<denom>")]
pub async fn get_balances_by_denom(
    app: &State<BaseApp>,
    addr: String,
    denom: String,
) -> Result<Json<QueryBalanceResponse>, Error> {
    let req = QueryBalanceRequest {
        address: AccAddress::from_bech32(&addr).map_err(|e| Error::bad_request(e.to_string()))?,
        denom: String::from(denom)
            .try_into()
            .map_err(|e: proto_types::Error| Error::bad_request(e.to_string()))?,
    };

    let store = app.multi_store.read().expect("RwLock will not be poisoned");
    let ctx = QueryContext::new(&store, app.get_block_height());

    Ok(Json(bank::Bank::query_balance(&ctx, req)))
}

// This is a hack for now to make the front end work
// TODO: remove this once the staking module is implemented
#[get("/cosmos/staking/v1beta1/params")]
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
