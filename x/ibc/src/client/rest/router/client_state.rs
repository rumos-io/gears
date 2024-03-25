use std::marker::PhantomData;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use gears::{
    application::ApplicationInfo,
    baseapp::{ABCIHandler, BaseApp, Genesis},
    client::rest::{error::Error, RestState},
    x::params::ParamsSubspaceKey,
};
use prost::Message as ProstMessage;
use proto_messages::cosmos::{
    ibc::{
        protobuf::Protobuf,
        query::response::QueryClientStateResponse,
        types::core::{
            client::context::types::proto::v1::QueryClientStateRequest, host::identifiers::ClientId,
        },
    },
    tx::v1beta1::message::Message,
};
use store::StoreKey;
use tendermint::abci::Application;
use tendermint::proto::abci::RequestQuery;

use crate::client::cli::query::client_state::STATE_URL;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct Route<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    M: Message,
    H: ABCIHandler<M, SK, G>,
    G: Genesis,
    AI: ApplicationInfo,
> {
    _sk: PhantomData<SK>,
    _psk: PhantomData<PSK>,
    _m: PhantomData<M>,
    _h: PhantomData<H>,
    _g: PhantomData<G>,
    _ai: PhantomData<AI>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        M: Message,
        H: ABCIHandler<M, SK, G>,
        G: Genesis,
        AI: ApplicationInfo,
    > Route<SK, PSK, M, H, G, AI>
{
    async fn handle(
        Path(client_id): Path<ClientId>,
        State(app): State<BaseApp<SK, PSK, M, H, G, AI>>,
    ) -> Result<Json<QueryClientStateResponse>, Error> {
        let query = QueryClientStateRequest {
            client_id: client_id.to_string(),
        };

        let request = RequestQuery {
            data: ProstMessage::encode_to_vec(&query).into(),
            path: STATE_URL.to_owned(),
            height: 0,
            prove: false,
        };

        let response = app.query(request);

        Ok(Json(
            QueryClientStateResponse::decode(response.value).map_err(|_| {
                Error::bad_gateway_with_msg("should be a valid QueryClientStateResponse".to_owned())
            })?,
        ))
    }

    pub fn router() -> Router<RestState<SK, PSK, M, H, G, AI>> {
        Router::new().route(
            constcat::concat!(STATE_URL, "/:client_id"),
            get(Self::handle),
        )
    }
}
