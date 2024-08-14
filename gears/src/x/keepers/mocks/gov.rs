use kv_store::StoreKey;

use crate::{
    types::{base::coin::UnsignedCoin, store::gas::errors::GasStoreErrors},
    x::{keepers::gov::GovernanceBankKeeper, module::Module},
};

use super::bank::MockBankKeeper;

impl<SK: StoreKey, M: Module> GovernanceBankKeeper<SK, M> for MockBankKeeper {
    // TODO: Move to BankKeeper
    fn balance_all<DB: database::Database, CTX: crate::context::QueryableContext<DB, SK>>(
        &self,
        _: &CTX,
        _: &address::AccAddress,
    ) -> Result<Vec<UnsignedCoin>, GasStoreErrors> {
        Ok(self.balance_all.clone())
    }

    fn balance<DB: database::Database, CTX: crate::context::QueryableContext<DB, SK>>(
        &self,
        _: &CTX,
        _: &address::AccAddress,
        _: &crate::types::denom::Denom,
    ) -> Result<UnsignedCoin, GasStoreErrors> {
        Ok(self.balance.clone())
    }
}
