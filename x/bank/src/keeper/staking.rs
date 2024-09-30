use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M> + Send + Sync + 'static,
        M: Module,
    > StakingBankKeeper<SK, M> for Keeper<SK, PSK, AK, M>
{
    fn send_coins_from_module_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_pool: &M,
        recepient_pool: &M,
        amount: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        self.send_coins_from_module_to_module(ctx, sender_pool, recepient_pool, amount)
    }

    fn undelegate_coins_from_module_to_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_module: &M,
        addr: AccAddress,
        amount: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        self.undelegate_coins_from_module_to_account(ctx, sender_module, addr, amount)
    }

    fn delegate_coins_from_account_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_addr: AccAddress,
        recepient_module: &M,
        amount: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        self.delegate_coins_from_account_to_module(ctx, sender_addr, recepient_module, amount)
    }
}
