use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M> + Send + Sync + 'static,
        M: Module + strum::IntoEnumIterator,
    > StakingBankKeeper<SK, M> for Keeper<SK, PSK, AK, M>
{
    fn undelegate_coins_from_module_to_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_module: &M,
        addr: AccAddress,
        amount: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        let sender_module_addr = sender_module.address();
        self.auth_keeper
            .check_create_new_module_account(ctx, sender_module)?;

        if !sender_module.permissions().iter().any(|p| p == "staking") {
            return Err(BankKeeperError::Permission(format!(
                "module account {} does not have permissions to receive undelegate coins",
                sender_module.name()
            )));
        }

        self.undelegate_coins(ctx, sender_module_addr, addr, amount)
    }

    fn delegate_coins_from_account_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_addr: AccAddress,
        recepient_module: &M,
        amount: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        let recepient_module_addr = recepient_module.address();
        self.auth_keeper
            .check_create_new_module_account(ctx, recepient_module)?;

        if !recepient_module
            .permissions()
            .iter()
            .any(|p| p == "staking")
        {
            return Err(BankKeeperError::Permission(format!(
                "module account {} does not have permissions to receive delegated coins",
                recepient_module.name()
            )));
        }
        self.delegate_coins(ctx, sender_addr, recepient_module_addr, amount)
    }
}
