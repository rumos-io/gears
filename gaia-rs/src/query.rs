use auth::cli::query::{AuthQuery, AuthQueryResponse};
use bank::cli::query::{BankQuery, BankQueryResponse};
use ibc::cli::client::query::{IbcQuery, IbcQueryResponse};
use proto_messages::cosmos::query::Query;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq)]
pub enum GaiaQuery {
    Auth(AuthQuery),
    Bank(BankQuery),
    Ibc(IbcQuery),
}

impl Query for GaiaQuery {
    fn query_url(&self) -> std::borrow::Cow<'static, str> {
        match self {
            GaiaQuery::Auth(var) => var.query_url(),
            GaiaQuery::Bank(var) => var.query_url(),
            GaiaQuery::Ibc(var) => var.query_url(),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            GaiaQuery::Auth(var) => var.into_bytes(),
            GaiaQuery::Bank(var) => var.into_bytes(),
            GaiaQuery::Ibc(var) => var.into_bytes(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum GaiaQueryResponse {
    Auth(AuthQueryResponse),
    Bank(BankQueryResponse),
    Ibc(IbcQueryResponse),
}
