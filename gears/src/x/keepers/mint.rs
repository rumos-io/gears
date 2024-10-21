use database::Database;
use kv_store::StoreKey;

use crate::{
    context::TransactionalContext,
    types::base::coins::UnsignedCoins,
    x::{errors::BankKeeperError, module::Module},
};

pub trait MintingBankKeeper<SK: StoreKey, M: Module>: Clone + Send + Sync + 'static {
    fn mint_coins<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        module: &M,
        amount: UnsignedCoins,
    ) -> Result<(), BankKeeperError>;
}
