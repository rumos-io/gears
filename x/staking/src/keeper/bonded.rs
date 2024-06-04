use super::*;
use gears::types::{account::ModuleAccount, store::errors::StoreErrors};

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    /// bonded_pool returns the bonded tokens pool's module account
    pub fn bonded_pool<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
    ) -> Option<ModuleAccount> {
        self.auth_keeper
            .module_account(ctx, BONDED_POOL_NAME.to_string())
    }

    /// bonded_pool returns the bonded tokens pool's module account
    pub fn not_bonded_pool<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
    ) -> Option<ModuleAccount> {
        self.auth_keeper
            .module_account(ctx, NOT_BONDED_POOL_NAME.to_string())
    }

    pub fn bonded_tokens_to_not_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        amount: Uint256,
    ) -> Result<(), StoreErrors> {
        let params = self.staking_params_keeper.try_get(ctx)?;

        // TODO: original routine is infallible, it means that the amount is a valid number.
        // The method is called from failable methods. Consider to provide correct solution taking
        // into account additional analisis.
        let coins = SendCoins::new(vec![Coin {
            denom: params.bond_denom,
            amount,
        }])
        .unwrap();

        // TODO: check and maybe remove unwrap
        self.bank_keeper
            .send_coins_from_module_to_module::<DB, AK, CTX>(
                ctx,
                BONDED_POOL_NAME.into(),
                NOT_BONDED_POOL_NAME.into(),
                coins,
            )
            .unwrap();

        Ok(())
    }

    pub fn bonded_to_unbonding<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        if validator.status != BondStatus::Bonded {
            return Err(AppError::Custom(format!(
                "bad state transition bonded to unbonding, validator: {}",
                validator.operator_address
            ))
            .into());
        }
        self.begin_unbonding_validator(ctx, validator)
    }

    pub fn bond_validator<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> Result<(), StoreErrors> {
        // delete the validator by power index, as the key will change
        self.delete_validator_by_power_index(ctx, validator)?;

        validator.update_status(BondStatus::Bonded);
        // save the now bonded validator record to the two referenced stores
        self.set_validator(ctx, validator)?;
        self.set_validator_by_power_index(ctx, validator)?;

        // delete from queue if present
        self.delete_validator_queue(ctx, validator)?;
        // trigger hook
        self.after_validator_bonded(ctx, validator);

        Ok(())
    }
}
