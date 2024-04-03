use database::{Database, PrefixDB};
use gears::{
    types::context::{QueryableContext, TransactionalContext},
    x::params::{Keeper, ParamsSubspaceKey},
};
use store::{ReadPrefixStore, StoreKey, WritePrefixStore};

pub const CLIENT_STATE_KEY: &str = "clientState";
pub const CLIENT_PARAMS_KEY: &str = "clientParams";
pub const NEXT_CLIENT_SEQUENCE: &str = "nextClientSequence";

#[derive(Debug, Clone)]
pub struct AbciParamsKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    pub params_keeper: Keeper<SK, PSK>,
    pub params_subspace_key: PSK,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("key should be set in kv store")]
pub struct ParamsError;

impl<SK: StoreKey, PSK: ParamsSubspaceKey> AbciParamsKeeper<SK, PSK> {
    pub fn get<DB: Database, CTX: QueryableContext<PrefixDB<DB>, SK>>(
        &self,
        ctx: &CTX,
        key: &impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, ParamsError> {
        let value = self
            .params_keeper
            .raw_subspace(ctx, &self.params_subspace_key)
            .get(key.as_ref())
            .ok_or(ParamsError)?;

        Ok(value)
    }

    pub fn set<DB: Database, CTX: TransactionalContext<PrefixDB<DB>, SK>>(
        &self,
        ctx: &mut CTX,
        key: impl IntoIterator<Item = u8>,
        value: impl IntoIterator<Item = u8>,
    ) {
        self.params_keeper
            .raw_subspace_mut(ctx, &self.params_subspace_key)
            .set(key, value);
    }
}
