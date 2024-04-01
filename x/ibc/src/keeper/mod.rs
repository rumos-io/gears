use database::Database;
use gears::{
    error::SearchError, types::context::read_context::ReadContext, x::params::ParamsSubspaceKey,
};
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
    kv_store_key::{KvStoreKey, SimpleKvStoreKey},
    StoreKey,
};

use crate::{
    params::{self, AbciParamsKeeper},
    types::ConsensusState,
};

pub mod query;
pub mod tx;

pub const KEY_CLIENT_STORE_PREFIX: &str = "clients";
pub const KEY_CONSENSUS_STATE_PREFIX: &str = "consensusStates";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ClientStoreKey(pub ClientId);

impl KvStoreKey for ClientStoreKey {
    fn prefix(self) -> store::kv_store_key::KeyBytes {
        format!("{KEY_CLIENT_STORE_PREFIX}/{}", self.0)
            .into_bytes()
            .try_into()
            .expect("Unreachable. `KEY_CLIENT_STORE_PREFIX` is not empty")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ConsensusStateKey(pub Height);

impl KvStoreKey for ConsensusStateKey {
    fn prefix(self) -> store::kv_store_key::KeyBytes {
        format!("{KEY_CONSENSUS_STATE_PREFIX}/{}", self.0)
            .into_bytes()
            .try_into()
            .expect("Unreachable. `KEY_CONSENSUS_STATE_PREFIX` is not empty")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ConsensusClientKey(pub ClientId);

impl KvStoreKey for ConsensusClientKey {
    fn prefix(self) -> store::kv_store_key::KeyBytes {
        format!("{KEY_CONSENSUS_STATE_PREFIX}/{}", self.0)
            .into_bytes()
            .try_into()
            .expect("Unreachable. `KEY_CONSENSUS_STATE_PREFIX` is not empty")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ConsensusOnRevisionHeightKey(pub u64);

impl KvStoreKey for ConsensusOnRevisionHeightKey {
    fn prefix(self) -> store::kv_store_key::KeyBytes {
        format!("{KEY_CONSENSUS_STATE_PREFIX}/{}", self.0)
            .into_bytes()
            .try_into()
            .expect("Unreachable. `KEY_CONSENSUS_STATE_PREFIX` is not empty")
    }
}

fn params_get<DB: Database, SK: StoreKey, PSK: ParamsSubspaceKey>(
    keeper: &AbciParamsKeeper<SK, PSK>,
    ctx: &impl ReadContext<SK, DB>,
) -> Result<Params, SearchError> {
    let bytes = keeper
        .get(
            ctx,
            SimpleKvStoreKey(
                params::CLIENT_PARAMS_KEY
                    .as_bytes()
                    .to_vec()
                    .try_into()
                    .expect("Unreachable. Const value should be valid"),
            ),
        )
        .map_err(|_| SearchError::NotFound)?;

    Ok(serde_json::from_slice::<RawParams>(&bytes)
        .map_err(|e| SearchError::DecodeError(e.to_string()))?
        .into())
}

pub fn client_state_get<DB: Database, SK: StoreKey>(
    store_key: &SK,
    ctx: &impl ReadContext<SK, DB>,
    key: ClientStoreKey,
) -> Result<WrappedTendermintClientState, SearchError> {
    let any_store = ctx.get_kv_store(store_key);
    let store: store::ImmutablePrefixStore<'_, database::PrefixDB<DB>> =
        any_store.get_immutable_prefix_store(key);

    let bytes = store
        .get(
            SimpleKvStoreKey::try_from(params::CLIENT_STATE_KEY.as_bytes().to_vec())
                .expect("Unreachable. `CLIENT_STATE_KEY` is not empty"),
        )
        .ok_or(SearchError::NotFound)?;

    let state = <WrappedTendermintClientState as Protobuf<PrimitiveAny>>::decode_vec(&bytes)
        .map_err(|e| SearchError::DecodeError(e.to_string()))?;

    Ok(state)
}

pub fn client_consensus_state<DB: Database, SK: StoreKey>(
    store_key: &SK,
    ctx: &impl ReadContext<SK, DB>,
    client_key: ClientStoreKey,
    consensus_key: ConsensusStateKey,
) -> Result<ConsensusState, SearchError> {
    let any_store = ctx.get_kv_store(store_key);
    let store = any_store.get_immutable_prefix_store(client_key);

    let bytes = store.get(consensus_key).ok_or(SearchError::NotFound)?;

    let state = <WrappedConsensusState as Protobuf<PrimitiveAny>>::decode_vec(&bytes)
        .map_err(|e| SearchError::DecodeError(e.to_string()))?;

    Ok(ConsensusState(state))
}
