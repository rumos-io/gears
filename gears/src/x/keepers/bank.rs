use database::Database;
use kv_store::StoreKey;

use crate::{
    error::AppError,
    types::{
        address::AccAddress,
        base::send::SendCoins,
        context::{QueryableContext, TransactionalContext},
        denom::Denom,
        tx::metadata::Metadata,
    },
    x::module::Module,
};

pub trait BankKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    fn send_coins_from_account_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        from_address: AccAddress,
        to_module: Module,
        amount: SendCoins,
    ) -> Result<(), AppError>;

    fn get_denom_metadata<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        base: &Denom,
    ) -> Option<Metadata>;
}
