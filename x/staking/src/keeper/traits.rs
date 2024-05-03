pub use super::*;

/// AccountKeeper defines the expected account keeper methods (noalias)
// TODO: AuthKeeper should implements module account stuff
pub trait AccountKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    // TODO: should be a sdk account interface
    fn get_account<DB: Database, AK: AuthKeeper<SK>, CTX: QueryableContext<DB, SK>>(
        _ctx: CTX,
        _addr: ValAddress,
    ) -> AK;

    // only used for simulation
    fn get_module_address(_name: String) -> ValAddress {
        todo!()
    }

    fn get_module_account<DB: Database, CTX: QueryableContext<DB, SK>>(
        _ctx: &CTX,
        _module_name: String,
    ) -> Self;

    fn set_module_account<DB: Database, AK: AuthKeeper<SK>, CTX: QueryableContext<DB, SK>>(
        _context: &CTX,
        _acc: AK,
    );
}

/// BankKeeper defines the expected interface needed to retrieve account balances.
pub trait BankKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    // GetAllBalances(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    // GetBalance(ctx sdk.Context, addr sdk.AccAddress, denom string) sdk.Coin
    // LockedCoins(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    // SpendableCoins(ctx sdk.Context, addr sdk.AccAddress) sdk.Coins
    //
    // GetSupply(ctx sdk.Context, denom string) sdk.Coin
    //
    // BurnCoins(ctx sdk.Context, name string, amt sdk.Coins) error

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
        // TODO: ConstAddr in cosmos sdk
        const_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn after_validator_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        // TODO: ConstAddr in cosmos sdk
        const_addr: AccAddress,
        val_addr: ValAddress,
    );

    fn after_validator_begin_unbonding<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        // TODO: ConstAddr in cosmos sdk
        const_addr: AccAddress,
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
        // TODO: original is an alias to bigint
        fraction: Decimal256,
    );
}
