use kv_store::StoreKey;

use crate::{
    types::{store::gas::errors::GasStoreErrors, tx::metadata::Metadata},
    x::{errors::BankKeeperError, keepers::bank::BankKeeper, module::Module},
};

#[derive(former::Former, Clone, Debug)]
pub struct MockBankKeeper {
    get_denom_metadata: Option<Metadata>,
}

impl<SK: StoreKey, M: Module> BankKeeper<SK, M> for MockBankKeeper {
    fn send_coins_from_account_to_module<
        DB: database::Database,
        CTX: crate::context::TransactionalContext<DB, SK>,
    >(
        &self,
        _: &mut CTX,
        _: address::AccAddress,
        _: &M,
        _: crate::types::base::coins::UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        Ok(())
    }

    fn send_coins_from_module_to_account<
        DB: database::Database,
        CTX: crate::context::TransactionalContext<DB, SK>,
    >(
        &self,
        _: &mut CTX,
        _: &address::AccAddress,
        _: &M,
        _: crate::types::base::coins::UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        Ok(())
    }

    fn get_denom_metadata<DB: database::Database, CTX: crate::context::QueryableContext<DB, SK>>(
        &self,
        _: &CTX,
        _: &crate::types::denom::Denom,
    ) -> Result<Option<Metadata>, GasStoreErrors> {
        Ok(self.get_denom_metadata.clone())
    }

    fn coins_burn<DB: database::Database, CTX: crate::context::TransactionalContext<DB, SK>>(
        &self,
        _: &mut CTX,
        _: &M,
        _: &crate::types::base::coins::UnsignedCoins,
    ) -> Result<(), crate::x::errors::BankKeeperError> {
        Ok(())
    }
}
