use gears::{
    params::ParamsSubspaceKey,
    store::StoreKey,
    tendermint::types::proto::event::{Event, EventAttribute},
    types::{base::coin::UnsignedCoin, uint::Uint256},
    x::{
        errors::{BankCoinsError, BankKeeperError},
        keepers::{auth::AuthKeeper, bank::BalancesKeeper, mint::MintingBankKeeper},
        module::Module,
    },
};

use super::Keeper;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        M: Module + strum::IntoEnumIterator,
    > MintingBankKeeper<SK, M> for Keeper<SK, PSK, AK, M>
{
    fn mint_coins<
        DB: gears::store::database::Database,
        CTX: gears::context::TransactionalContext<DB, SK>,
    >(
        &self,
        ctx: &mut CTX,
        module: &M,
        amount: gears::types::base::coins::UnsignedCoins,
    ) -> Result<(), gears::x::errors::BankKeeperError> {
        if !module
            .permissions()
            .iter()
            .any(|this| this.as_str() == "minter")
        {
            Err(BankKeeperError::ModulePermission)?
        }

        let module_addr = module.address();

        self.add_coins(ctx, &module_addr, amount.inner())?;

        let event = Event::new(
            "coinbase",
            [
                EventAttribute::new("minter".into(), String::from(module_addr).into(), true),
                EventAttribute::new(
                    "amount".into(),
                    gears::types::base::coins::format_coins(amount.inner()),
                    true,
                ),
            ],
        );

        for UnsignedCoin { denom, amount } in amount {
            let mut supply = match self.supply(ctx, &denom)? {
                Some(supply) => supply,
                None => UnsignedCoin {
                    denom,
                    amount: Uint256::zero(),
                },
            };

            supply.amount = supply
                .amount
                .checked_add(amount)
                .map_err(|e| BankKeeperError::Coins(BankCoinsError::Math(e.to_string())))?;

            self.set_supply(ctx, supply)?;
        }

        ctx.push_event(event);

        Ok(())
    }
}
