use database::Database;
use gears::{error::SearchError, types::context::Context, x::params::ParamsSubspaceKey};
use proto_messages::{
    any::PrimitiveAny,
    cosmos::ibc::{
        protobuf::Protobuf,
        types::{
            core::{
                client::{
                    proto::RawParams,
                    types::{Height, Params},
                },
                host::identifiers::ClientId,
            },
            tendermint::{consensus_state::WrappedConsensusState, WrappedTendermintClientState},
        },
    },
};
use store::{
    types::prefix::immutable::ImmutablePrefixStore, ReadKVStore, ReadPrefixStore, StoreKey,
};

use crate::{
    params::{self, AbciParamsKeeper},
    types::ConsensusState,
};

pub mod query;
pub mod tx;

pub const KEY_CLIENT_STORE_PREFIX: &str = "clients";
pub const KEY_CONSENSUS_STATE_PREFIX: &str = "consensusStates";

fn params_get<DB: Database, SK: StoreKey, PSK: ParamsSubspaceKey>(
    keeper: &AbciParamsKeeper<SK, PSK>,
    ctx: &impl Context<DB, SK>,
) -> Result<Params, SearchError> {
    let bytes = keeper
        .get(ctx, &params::CLIENT_PARAMS_KEY)
        .map_err(|_| SearchError::NotFound)?;

    Ok(serde_json::from_slice::<RawParams>(&bytes)
        .map_err(|e| SearchError::DecodeError(e.to_string()))?
        .into())
}

pub fn client_state_get<DB: Database, SK: StoreKey>(
    store_key: &SK,
    ctx: &impl Context<DB, SK>,
    client_id: &ClientId,
) -> Result<WrappedTendermintClientState, SearchError> {
    let any_store = ctx.kv_store(store_key);
    let store: ImmutablePrefixStore<'_, database::PrefixDB<DB>> =
        any_store.prefix_store(format!("{KEY_CLIENT_STORE_PREFIX}/{client_id}").into_bytes());

    let bytes = store
        .get(params::CLIENT_STATE_KEY.as_bytes())
        .ok_or(SearchError::NotFound)?;

    let state = <WrappedTendermintClientState as Protobuf<PrimitiveAny>>::decode_vec(&bytes)
        .map_err(|e| SearchError::DecodeError(e.to_string()))?;

    Ok(state)
}

pub fn client_consensus_state<DB: Database, SK: StoreKey>(
    store_key: &SK,
    ctx: &impl Context<DB, SK>,
    client_id: &ClientId,
    height: &Height,
) -> Result<ConsensusState, SearchError> {
    let any_store = ctx.kv_store(store_key);
    let store =
        any_store.prefix_store(format!("{KEY_CLIENT_STORE_PREFIX}/{client_id}").into_bytes());

    let bytes = store
        .get(format!("{KEY_CONSENSUS_STATE_PREFIX}/{height}").as_bytes())
        .ok_or(SearchError::NotFound)?;

    let state = <WrappedConsensusState as Protobuf<PrimitiveAny>>::decode_vec(&bytes)
        .map_err(|e| SearchError::DecodeError(e.to_string()))?;

    Ok(ConsensusState(state))
}
