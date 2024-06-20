use database::Database;
use kv_store::StoreKey;

use crate::{
    context::QueryableContext,
    types::{address::AccAddress, store::gas::errors::GasStoreErrors},
    x::{
        module::Module,
        types::{delegation::StakingDelegation, validator::StakingValidator},
    },
};

pub trait StakingKeeper<SK: StoreKey, M: Module>: Clone + Send + Sync + 'static {
    type Validator: StakingValidator;
    type Delegation: StakingDelegation;

    fn bonded_validators_by_power<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<impl Iterator<Item = Result<Self::Validator, GasStoreErrors>>, GasStoreErrors>;

    fn delegations<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        voter: &AccAddress,
    ) -> impl Iterator<Item = Result<Self::Delegation, GasStoreErrors>>;
}
