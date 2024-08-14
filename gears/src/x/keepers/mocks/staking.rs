use kv_store::StoreKey;

use crate::{
    context::TransactionalContext,
    types::{
        base::{coin::UnsignedCoin, coins::UnsignedCoins},
        store::gas::errors::GasStoreErrors,
    },
    x::{errors::BankKeeperError, keepers::staking::StakingBankKeeper, module::Module},
};

use super::bank::MockBankKeeper;

impl<SK: StoreKey, M: Module> StakingBankKeeper<SK, M> for MockBankKeeper {
    //  TODO: Remove and use balances_all
    fn get_all_balances<DB: database::Database, CTX: crate::context::QueryableContext<DB, SK>>(
        &self,
        _: &CTX,
        _: address::AccAddress,
    ) -> Result<Vec<UnsignedCoin>, GasStoreErrors> {
        Ok(self.balance_all.clone())
    }

    fn send_coins_from_module_to_module<
        DB: database::Database,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        _: &mut CTX,
        _: &M,
        _: &M,
        _: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        Ok(())
    }

    fn undelegate_coins_from_module_to_account<
        DB: database::Database,
        CTX: crate::context::TransactionalContext<DB, SK>,
    >(
        &self,
        _: &mut CTX,
        _: &M,
        _: address::AccAddress,
        _: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        Ok(())
    }

    fn delegate_coins_from_account_to_module<
        DB: database::Database,
        CTX: TransactionalContext<DB, SK>,
    >(
        &self,
        _: &mut CTX,
        _: address::AccAddress,
        _: &M,
        _: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        Ok(())
    }
}
