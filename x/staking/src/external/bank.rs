use gears::{
    context::{QueryableContext, TransactionalContext},
    error::AppError,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    types::{
        address::AccAddress,
        base::{coin::Coin, send::SendCoins},
        store::gas::errors::GasStoreErrors,
    },
    x::{keepers::auth::AuthKeeper, module::Module},
};

use crate::BankKeeper;

impl<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK, M>, M: Module> BankKeeper<SK, M>
    for bank::Keeper<SK, PSK, AK, M>
{
    fn get_all_balances<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: AccAddress,
    ) -> Result<Vec<Coin>, GasStoreErrors> {
        self.get_all_balances(ctx, addr)
    }

    fn send_coins_from_module_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_pool: &M,
        recepient_pool: &M,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        self.send_coins_from_module_to_module(ctx, sender_pool, recepient_pool, amount)
    }

    fn undelegate_coins_from_module_to_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_module: &M,
        addr: AccAddress,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        self.undelegate_coins_from_module_to_account(ctx, sender_module, addr, amount)
    }

    fn delegate_coins_from_account_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        sender_addr: AccAddress,
        recepient_module: &M,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        self.delegate_coins_from_account_to_module(ctx, sender_addr, recepient_module, amount)
    }
}
