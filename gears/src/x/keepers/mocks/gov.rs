use kv_store::StoreKey;

use crate::{
    types::{base::coin::UnsignedCoin, store::gas::errors::GasStoreErrors},
    x::{keepers::gov::GovernanceBankKeeper, module::Module},
};

use super::bank::MockBankKeeper;

impl<SK: StoreKey, M: Module> GovernanceBankKeeper<SK, M> for MockBankKeeper {
    fn balance<DB: database::Database, CTX: crate::context::QueryableContext<DB, SK>>(
        &self,
        _: &CTX,
        _: &address::AccAddress,
        _: &crate::types::denom::Denom,
    ) -> Result<UnsignedCoin, GasStoreErrors> {
        Ok(self.balance.clone().expect("You need to set balance"))
    }
}
