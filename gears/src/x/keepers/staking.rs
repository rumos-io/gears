use database::Database;
use kv_store::StoreKey;

use crate::{
    context::QueryableContext,
    error::AppError,
    types::{address::AccAddress, base::coin::Coin, store::gas::errors::GasStoreErrors},
    x::{
        module::Module,
        types::{delegation::StakingDelegation, validator::StakingValidator},
    },
};

/// Staking keeper which used in gov xmod
pub trait GovStakingKeeper<SK: StoreKey, M: Module>: Clone + Send + Sync + 'static {
    type Validator: StakingValidator;
    type Delegation: StakingDelegation;

    fn bonded_validators_by_power_iter<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<impl Iterator<Item = Result<Self::Validator, GasStoreErrors>>, GasStoreErrors>;

    fn delegations_iter<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        voter: &AccAddress,
    ) -> impl Iterator<Item = Result<Self::Delegation, GasStoreErrors>>;

    fn total_bonded_tokens<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Coin, AppError>;
}
