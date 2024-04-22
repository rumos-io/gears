use gears::store::ReadPrefixStore;
use gears::store::WritePrefixStore;
use gears::{
    params::ParamsSubspaceKey,
    store::{
        database::{Database, PrefixDB},
        StoreKey,
    },
    types::context::{QueryableContext, TransactionalContext},
};
use serde::{Deserialize, Serialize};

const KEY_ALLOWED_CLIENTS: &[u8; 14] = b"AllowedClients";

/// Params defines the set of IBC light client parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct Params {
    /// allowed_clients defines the list of allowed client state types which can be created
    /// and interacted with. If a client type is removed from the allowed clients list, usage
    /// of this client will be disabled until it is added again to the list.
    #[prost(string, repeated, tag = "1")]
    pub allowed_clients: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClientParamsKeeper<SK, PSK> {
    pub params_keeper: gears::params::Keeper<SK, PSK>,
    pub params_subspace_key: PSK,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> ClientParamsKeeper<SK, PSK> {
    pub fn _get<DB: Database, CTX: QueryableContext<PrefixDB<DB>, SK>>(&self, ctx: &CTX) -> Params {
        let store = self
            .params_keeper
            .raw_subspace(ctx, &self.params_subspace_key);

        let raw_allowed_clients: String = String::from_utf8(
            store
                .get(KEY_ALLOWED_CLIENTS)
                .expect("key should be set in kv store")
                .clone(),
        )
        .expect("should be valid utf-8");

        let allowed_clients: Vec<String> =
            serde_json::from_str(&raw_allowed_clients).expect("conversion from json won't fail");

        Params { allowed_clients }
    }

    pub fn set<DB: Database, CTX: TransactionalContext<PrefixDB<DB>, SK>>(
        &self,
        ctx: &mut CTX,
        params: Params,
    ) {
        let mut store = self
            .params_keeper
            .raw_subspace_mut(ctx, &self.params_subspace_key);

        let params =
            serde_json::to_string(&params.allowed_clients).expect("conversion to json won't fail");
        store.set(KEY_ALLOWED_CLIENTS.to_owned(), params.into_bytes());
    }
}
