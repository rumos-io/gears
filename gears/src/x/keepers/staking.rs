use database::Database;
use kv_store::StoreKey;

use crate::{
    context::QueryableContext,
    types::store::gas::errors::GasStoreErrors,
    x::{module::Module, types::validator::StakingValidator},
};

pub trait StakingKeeper<SK: StoreKey, M: Module>: Clone + Send + Sync + 'static {
    type Validator: StakingValidator;

    fn bonded_validators_by_power<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<impl Iterator<Item = Result<Self::Validator, GasStoreErrors>>, GasStoreErrors>;
}
