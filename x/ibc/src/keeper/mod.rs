use database::Database;
use gears::{
    types::context::{context::Context, read_context::ReadContext},
    x::params::ParamsSubspaceKey,
};
use proto_messages::{
    any::PrimitiveAny,
    cosmos::ibc::{
        protobuf::Protobuf,
        types::{
            core::{
                client::{proto::RawParams, types::Params},
                host::identifiers::ClientId,
            },
            tendermint::WrappedTendermintClientState,
        },
    },
};
use store::StoreKey;

use crate::{
    errors::SearchError,
    params::{self, AbciParamsKeeper},
};

pub mod query;
pub mod tx;

pub const KEY_CLIENT_STORE_PREFIX: &str = "clients";

pub(self) fn params_get<DB: Database, SK: StoreKey, PSK: ParamsSubspaceKey>(
    keeper: &AbciParamsKeeper<SK, PSK>,
    ctx: Context<'_, '_, DB, SK>,
) -> Result<Params, SearchError> {
    let bytes = keeper
        .get(&ctx, &params::CLIENT_PARAMS_KEY)
        .map_err(|_| SearchError::NotFound)?;

    Ok(serde_json::from_slice::<RawParams>(&bytes)
        .map_err(|e| SearchError::DecodeError(e.to_string()))?
        .into())
}

pub(self) fn client_state_get<DB: Database, SK: StoreKey>(
    store_key: &SK,
    ctx: &impl ReadContext<SK, DB>,
    client_id: &ClientId,
) -> Result<WrappedTendermintClientState, SearchError> {
    // TODO: Unsure in this code https://github.com/cosmos/ibc-go/blob/41e7bf14f717d5cc2815688c8c590769ed164389/modules/core/02-client/keeper/keeper.go#L78
    let any_store = ctx.get_kv_store(store_key);
    let store = any_store
        .get_immutable_prefix_store(format!("{KEY_CLIENT_STORE_PREFIX}/{client_id}").into_bytes());

    let bytes = store
        .get(params::CLIENT_STATE_KEY.as_bytes())
        .ok_or(SearchError::NotFound)?;

    let state = <WrappedTendermintClientState as Protobuf<PrimitiveAny>>::decode_vec(&bytes)
        .map_err(|e| SearchError::DecodeError(e.to_string()))?;

    Ok(state)
}
