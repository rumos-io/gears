use auth::cli::{
    query::{AuthQuery, AuthQueryResponse},
    tx_query::{TxQuery, TxQueryResponse},
};
use bank::cli::query::{BankQuery, BankQueryResponse};
use gears::{
    baseapp::{Query, QueryResponse},
    types::tx::TxMessage,
};
use ibc_rs::client::cli::query::{IbcQuery, IbcQueryResponse};
use serde::{Deserialize, Serialize};
use staking::cli::query::{StakingQuery, StakingQueryResponse};

#[derive(Clone, PartialEq)]
pub enum GaiaQuery {
    Auth(AuthQuery),
    Bank(BankQuery),
    Staking(StakingQuery),
    Ibc(IbcQuery),
    Tx(TxQuery),
    Txs(TxQuery),
}

impl Query for GaiaQuery {
    fn query_url(&self) -> &'static str {
        match self {
            GaiaQuery::Auth(var) => var.query_url(),
            GaiaQuery::Bank(var) => var.query_url(),
            GaiaQuery::Staking(var) => var.query_url(),
            GaiaQuery::Ibc(var) => var.query_url(),
            GaiaQuery::Tx(var) => var.query_url(),
            GaiaQuery::Txs(var) => var.query_url(),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            GaiaQuery::Auth(var) => var.into_bytes(),
            GaiaQuery::Bank(var) => var.into_bytes(),
            GaiaQuery::Staking(var) => var.into_bytes(),
            GaiaQuery::Ibc(var) => var.into_bytes(),
            GaiaQuery::Tx(var) => var.into_bytes(),
            GaiaQuery::Txs(var) => var.into_bytes(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum GaiaQueryResponse<M: TxMessage> {
    Auth(AuthQueryResponse),
    Bank(BankQueryResponse),
    Staking(StakingQueryResponse),
    Ibc(IbcQueryResponse),
    Tx(TxQueryResponse<M>),
}

impl<M: TxMessage> QueryResponse for GaiaQueryResponse<M> {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            GaiaQueryResponse::Auth(msg) => msg.into_bytes(),
            GaiaQueryResponse::Bank(msg) => msg.into_bytes(),
            GaiaQueryResponse::Staking(msg) => msg.into_bytes(),
            GaiaQueryResponse::Ibc(msg) => msg.into_bytes(),
            GaiaQueryResponse::Tx(msg) => msg.into_bytes(),
        }
    }
}
