use super::*;
use gears::{
    types::account::{Account, ModuleAccount},
    types::address::{AccAddress, ConsAddress},
};

/// AccountKeeper defines the expected account keeper methods (noalias)
pub trait AccountKeeper<SK: StoreKey>: AuthKeeper<SK> + Clone + Send + Sync + 'static {
    fn account<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: CTX,
        addr: ValAddress,
    ) -> Account;

    fn module_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        module_name: String,
    ) -> Option<ModuleAccount>;

    fn set_module_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        context: &mut CTX,
        acc: ModuleAccount,
    );
}

/// BankKeeper defines the expected interface needed to retrieve account balances.
pub trait BankKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    // GetBalance(ctx sdk.Context, addr sdk.AccAddress, denom string) sdk.Coin
    // LockedCoins(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    // SpendableCoins(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    //
    // GetSupply(ctx sdk.Context, denom string) sdk.Coin
    //
    // BurnCoins(ctx sdk.Context, name string, amt sdk.Coins) error

    fn all_balances<DB: Database, AK: AccountKeeper<SK>, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        addr: AccAddress,
    ) -> SendCoins;

    fn send_coins_from_module_to_module<
        DB: Database,
        AK: AccountKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        sender_pool: String,
        recepient_pool: String,
        amount: SendCoins,
    ) -> Result<(), AppError>;

    fn undelegate_coins_from_module_to_account<
        DB: Database,
        AK: AccountKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        sender_module: String,
        addr: AccAddress,
        amount: SendCoins,
    ) -> Result<(), AppError>;

    fn delegate_coins_from_account_to_module<
        DB: Database,
        AK: AccountKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        sender_addr: AccAddress,
        recepient_module: String,
        amount: SendCoins,
    ) -> Result<(), AppError>;
}

/// Event Hooks
/// These can be utilized to communicate between a staking keeper and another
/// keeper which must take particular actions when validators/delegators change
/// state. The second keeper must implement this interface, which then the
/// staking keeper can call.
pub trait KeeperHooks<SK: StoreKey>: Clone + Send + Sync + 'static {
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

    fn before_delegation_shares_modified<
        DB: Database,
        AK: AuthKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        del_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn before_delegation_removed<
        DB: Database,
        AK: AuthKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
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

    fn before_validator_slashed<
        DB: Database,
        AK: AuthKeeper<SK>,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        val_addr: ValAddress,
        fraction: Decimal256,
    );
}
