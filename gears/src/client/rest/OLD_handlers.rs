use bytes::Bytes;
use ibc_proto::cosmos::base::query::v1beta1::PageResponse;
use std::hash::Hash;
use store_crate::StoreKey;
use strum::IntoEnumIterator;

use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::bank::v1beta1::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest, QueryBalanceResponse,
    QueryTotalSupplyResponse,
};

use proto_messages::cosmos::base::abci::v1beta1::TxResponse;
use proto_messages::cosmos::tx::v1beta1::{AnyTx, GetTxsEventResponse, Message, Tx};
use proto_types::AccAddress;
use rocket::State;
use rocket::{get, serde::json::Json};
use serde::{Deserialize, Serialize};
use tendermint_informal::node::Info;
use tendermint_rpc::{endpoint::tx_search::Response, query::Query, Order};
use tendermint_rpc::{Client, HttpClient};

use super::pagination::parse_pagination;
use crate::TM_ADDRESS;
//use crate::baseapp::BaseApp;
use crate::client::rest::{error::Error, pagination::Pagination};

//#########################################

// /// Get all balances for a given address
// #[get("/cosmos/bank/v1beta1/balances/<addr>?<pagination>")]
// #[allow(unused_variables)]
// pub async fn get_balances(
//     app: &State<BaseApp>,
//     addr: String,
//     pagination: Pagination,
// ) -> Result<Json<QueryAllBalancesResponse>, Error> {
//     let req = QueryAllBalancesRequest {
//         address: AccAddress::from_bech32(&addr).map_err(|e| Error::bad_request(e.to_string()))?,
//         pagination: None,
//     };

//     let store = app.multi_store.read().expect("RwLock will not be poisoned");
//     let ctx = QueryContext::new(&store, app.get_block_height());

//     Ok(Json(bank::Bank::query_all_balances(&ctx, req)))
// }

// /// Get balance for a given address and denom
// #[get("/cosmos/bank/v1beta1/balances/<addr>/by_denom?<denom>")]
// pub async fn get_balances_by_denom(
//     app: &State<BaseApp>,
//     addr: String,
//     denom: String,
// ) -> Result<Json<QueryBalanceResponse>, Error> {
//     let req = QueryBalanceRequest {
//         address: AccAddress::from_bech32(&addr).map_err(|e| Error::bad_request(e.to_string()))?,
//         denom: String::from(denom)
//             .try_into()
//             .map_err(|e: proto_types::Error| Error::bad_request(e.to_string()))?,
//     };

//     let store = app.multi_store.read().expect("RwLock will not be poisoned");
//     let ctx = QueryContext::new(&store, app.get_block_height());

//     Ok(Json(bank::Bank::query_balance(&ctx, req)))
// }
