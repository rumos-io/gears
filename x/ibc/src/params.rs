use gears::params::ParamsSubspaceKey;
use gears::store::database::{Database, PrefixDB};
use gears::store::{
    QueryableMultiKVStore, ReadPrefixStore, StoreKey, TransactionalMultiKVStore, WritePrefixStore,
};
use gears::types::context::{QueryableContext, TransactionalContext};

pub const CLIENT_STATE_KEY: &str = "clientState";
pub const CLIENT_PARAMS_KEY: &str = "clientParams";
pub const NEXT_CLIENT_SEQUENCE: &str = "nextClientSequence";

#[derive(Debug, Clone)]
pub struct IBCParamsKeeper<SK, PSK> {
    pub params_keeper: gears::params::Keeper<SK, PSK>,
    pub params_subspace_key: PSK,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("key should be set in kv store")]
pub struct ParamsError;

impl<SK: StoreKey, PSK: ParamsSubspaceKey> IBCParamsKeeper<SK, PSK> {
    pub fn get<DB: Database, KV: QueryableMultiKVStore<DB, SK>>(
        &self,
        ctx: &KV,
        key: &impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, ParamsError> {
        let value = self
            .params_keeper
            .raw_subspace(ctx, &self.params_subspace_key)
            .get(key.as_ref())
            .ok_or(ParamsError)?;

        Ok(value)
    }

    pub fn set<DB: Database, KV: TransactionalMultiKVStore<DB, SK>>(
        &self,
        ctx: &mut KV,
        key: impl IntoIterator<Item = u8>,
        value: impl IntoIterator<Item = u8>,
    ) {
        self.params_keeper
            .raw_subspace_mut(ctx, &self.params_subspace_key)
            .set(key, value);
    }
}
