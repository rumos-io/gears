use kv_store::StoreKey;

use crate::{
    context::TransactionalContext,
    types::base::coins::UnsignedCoins,
    x::{errors::BankKeeperError, keepers::staking::StakingBankKeeper, module::Module},
};

use super::bank::MockBankKeeper;

impl<SK: StoreKey, M: Module> StakingBankKeeper<SK, M> for MockBankKeeper {
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
