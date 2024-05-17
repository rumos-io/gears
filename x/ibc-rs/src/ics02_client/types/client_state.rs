use derive_more::{From, TryInto};
//use gears::core::Protobuf;
use gears::params::ParamsSubspaceKey;
use gears::store::database::Database;
use gears::store::StoreKey;

use ibc::clients::tendermint::client_state::ClientState as TmClientState;
use ibc::clients::tendermint::types::{
    ClientState as ClientStateType, TENDERMINT_CLIENT_STATE_TYPE_URL,
};
use ibc::core::client::types::error::ClientError;
use ibc::derive::ClientState;
use ibc::primitives::proto::{Any, Protobuf};
use serde::{Deserialize, Serialize};

use crate::types::context::Context;

#[derive(ClientState, Clone, From, TryInto, Debug, Serialize, PartialEq, Deserialize)]
#[validation(Context<'a, 'b , DB: Database, SK: StoreKey, PSK:ParamsSubspaceKey >)]
#[execution(Context<'a, 'b, DB: Database, SK:StoreKey, PSK: ParamsSubspaceKey>)]
// TODO: this enum doesn't serialize to the same JSON as the Cosmos SDK. This is a separate issue to the derive macro issue.
// E.g. durations are serialized as fields rather than strings, some fields are in camleCase rather than snake_case.
//#[serde(tag = "@type")] // TODO: uncommenting this causes ClientState derive macro to panic. Uncomment when fixed
pub enum ClientState {
    //#[serde(rename = "/ibc.lightclients.tendermint.v1.ClientState")] // TODO: uncomment when above TODO is fixed
    Tendermint(TmClientState),
}

impl From<ClientStateType> for ClientState {
    fn from(value: ClientStateType) -> Self {
        ClientState::Tendermint(value.into())
    }
}

impl TryFrom<ClientState> for ClientStateType {
    type Error = ClientError;

    fn try_from(value: ClientState) -> Result<Self, Self::Error> {
        match value {
            ClientState::Tendermint(tm_client_state) => Ok(tm_client_state.inner().clone()),
        }
    }
}

impl From<ClientState> for Any {
    fn from(value: ClientState) -> Self {
        match value {
            ClientState::Tendermint(tm_client_state) => tm_client_state.into(),
        }
    }
}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            TENDERMINT_CLIENT_STATE_TYPE_URL => Ok(ClientState::Tendermint(value.try_into()?)),
            _ => Err(ClientError::Other {
                description: "Unknown client state type".into(),
            }),
        }
    }
}

impl Protobuf<Any> for ClientState {}
