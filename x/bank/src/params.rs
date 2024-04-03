use database::Database;
use gears::{
    types::context::{Context, ContextMut},
    x::params::ParamsSubspaceKey,
};
use serde::{Deserialize, Serialize};
use store::{ReadPrefixStore, StoreKey, WritePrefixStore};

const KEY_SEND_ENABLED: [u8; 11] = [083, 101, 110, 100, 069, 110, 097, 098, 108, 101, 100]; // "SendEnabled"
const KEY_DEFAULT_SEND_ENABLED: [u8; 18] = [
    068, 101, 102, 097, 117, 108, 116, 083, 101, 110, 100, 069, 110, 097, 098, 108, 101, 100,
]; // "DefaultSendEnabled"

// NOTE: The send_enabled field of the bank params is hard coded to the empty list for now
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Params {
    pub default_send_enabled: bool,
}

pub const DEFAULT_PARAMS: Params = Params {
    default_send_enabled: true,
};

#[derive(Debug, Clone)]
pub struct BankParamsKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    pub params_keeper: gears::x::params::Keeper<SK, PSK>,
    pub params_subspace_key: PSK,
}

// TODO: add a macro to create this?
impl<SK: StoreKey, PSK: ParamsSubspaceKey> BankParamsKeeper<SK, PSK> {
    pub fn get<DB: Database, CTX: Context<DB, SK>>(&self, ctx: &CTX) -> Params {
        let store = self
            .params_keeper
            .raw_subspace(ctx, &self.params_subspace_key);

        let default_send_enabled: bool = String::from_utf8(
            store
                .get(&KEY_DEFAULT_SEND_ENABLED)
                .expect("key should be set in kv store")
                .clone(),
        )
        .expect("should be valid utf-8")
        .parse()
        .expect("should be valid bool");

        Params {
            default_send_enabled,
        }
    }

    pub fn set<DB: Database, CTX: ContextMut<DB, SK>>(&self, ctx: &mut CTX, params: Params) {
        // let store = ctx.get_mutable_kv_store(crate::store::Store::Params);
        // let mut store = store.get_mutable_prefix_store(SUBSPACE_NAME.into());

        let mut store = self
            .params_keeper
            .raw_subspace_mut(ctx, &self.params_subspace_key);

        store.set(
            KEY_DEFAULT_SEND_ENABLED,
            params.default_send_enabled.to_string().into_bytes(),
        );

        // The send_enabled field is hard coded to the empty list for now
        store.set(KEY_SEND_ENABLED, "[]".bytes());
    }
}
