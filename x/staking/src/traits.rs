use gears::{
    context::{QueryableContext, TransactionalContext},
    error::AppError,
    store::{database::Database, StoreKey},
    types::{
        address::{AccAddress, ConsAddress, ValAddress},
        base::{coin::Coin, send::SendCoins},
        decimal256::Decimal256,
        store::gas::errors::GasStoreErrors,
    },
    x::{keepers::auth::AuthKeeper, module::Module},
};

/// BankKeeper defines the expected interface needed to retrieve account balances.
pub trait BankKeeper<SK: StoreKey, M: Module>: Clone + Send + Sync + 'static {
    // GetBalance(ctx sdk.Context, addr sdk.AccAddress, denom string) sdk.Coin
    // LockedCoins(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    // SpendableCoins(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    //
    // GetSupply(ctx sdk.Context, denom string) sdk.Coin
    //
    // BurnCoins(ctx sdk.Context, name string, amt sdk.Coins) error

    fn get_all_balances<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: AccAddress,
    ) -> Result<Vec<Coin>, GasStoreErrors>;

    fn send_coins_from_module_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_pool: &M,
        recepient_pool: &M,
        amount: SendCoins,
    ) -> Result<(), AppError>;

    fn undelegate_coins_from_module_to_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_module: &M,
        addr: AccAddress,
        amount: SendCoins,
    ) -> Result<(), AppError>;

    fn delegate_coins_from_account_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_addr: AccAddress,
        recepient_module: &M,
        amount: SendCoins,
    ) -> Result<(), AppError>;
}

/// Event Hooks
/// These can be utilized to communicate between a staking keeper and another
/// keeper which must take particular actions when validators/delegators change
/// state. The second keeper must implement this interface, which then the
/// staking keeper can call.
pub trait KeeperHooks<SK: StoreKey, AK: AuthKeeper<SK, M>, M: Module>:
    Clone + Send + Sync + 'static
{
    fn after_validator_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        val_addr: ValAddress,
    );

    fn before_validator_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        val_addr: ValAddress,
    );

    fn after_validator_removed<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        cons_addr: ConsAddress,
        val_addr: ValAddress,
    );

    fn after_validator_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        cons_addr: ConsAddress,
        val_addr: ValAddress,
    );

    fn after_validator_begin_unbonding<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        cons_addr: ConsAddress,
        val_addr: ValAddress,
    );

    fn before_delegation_created<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn before_delegation_shares_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn before_delegation_removed<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn after_delegation_modified<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn before_validator_slashed<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        val_addr: ValAddress,
        fraction: Decimal256,
    );
}
