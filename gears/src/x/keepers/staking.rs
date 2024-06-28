use cosmwasm_std::Decimal256;
use database::Database;
use kv_store::StoreKey;

use crate::{
    context::{QueryableContext, TransactionalContext},
    types::{
        address::{AccAddress, ConsAddress, ValAddress},
        base::coin::Coin,
        store::gas::errors::GasStoreErrors,
    },
    x::{
        module::Module,
        types::{delegation::StakingDelegation, validator::StakingValidator},
    },
};

/// Delay, in blocks, between when validator updates are returned to the
/// consensus-engine and when they are applied. For example, if
/// ValidatorUpdateDelay is set to X, and if a validator set update is
/// returned with new validators at the end of block 10, then the new
/// validators are expected to sign blocks beginning at block 11+X.
///
/// This value is constant as this should not change without a hard fork.
/// For Tendermint this should be set to 1 block, for more details see:
/// https://tendermint.com/docs/spec/abci/apps.html#endblock
// TODO: may it be u32?
pub const VALIDATOR_UPDATE_DELAY: i64 = 1;

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
    ) -> Result<Coin, GasStoreErrors>;
}

/// Staking keeper which used in slashing xmod
pub trait SlashingStakingKeeper<SK: StoreKey, M: Module>: Clone + Send + Sync + 'static {
    type Validator: StakingValidator;
    type Delegation: StakingDelegation;

    /// iterate through validators by operator address, execute func for each validator
    fn validators_iter<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<impl Iterator<Item = Result<Self::Validator, GasStoreErrors>>, GasStoreErrors>;

    /// get a particular validator by operator address
    fn validator<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &ValAddress,
    ) -> Result<Option<Self::Validator>, GasStoreErrors>;

    /// get a particular validator by consensus address
    fn validator_by_cons_addr<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: &ConsAddress,
    ) -> Result<Option<Self::Validator>, GasStoreErrors>;

    /// slash the validator and delegators of the validator, specifying offence height, offence power, and slash fraction
    fn slash<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &ConsAddress,
        height: u32,
        power: u32,
        slash_fraction_downtime: Decimal256,
    ) -> Result<(), GasStoreErrors>;

    /// jail a validator
    fn jail<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &ConsAddress,
    ) -> Result<(), GasStoreErrors>;

    /// unjail a validator
    fn unjail<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: &ConsAddress,
    ) -> Result<(), GasStoreErrors>;

    /// delegation allows for getting a particular delegation for a given validator
    /// and delegator outside the scope of the staking module.
    fn delegation<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        delegator_address: &AccAddress,
        validator_address: &ValAddress,
    ) -> Result<Option<Self::Delegation>, GasStoreErrors>;

    /// max_validators returns the maximum amount of bonded validators
    fn max_validators<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<u32, GasStoreErrors>;
    // MaxValidators(sdk.Context) uint32
}
