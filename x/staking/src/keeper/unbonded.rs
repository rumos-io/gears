pub use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AccountKeeper<SK>,
        BK: BankKeeper<SK>,
        KH: KeeperHooks<SK>,
    > Keeper<SK, PSK, AK, BK, KH>
{
    pub fn unbonded_to_bonded<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        validator: &mut Validator,
    ) -> anyhow::Result<()> {
        if validator.status != BondStatus::Unbonded {
            return Err(AppError::Custom(format!(
                "bad state transition unbonded to bonded, validator: {}",
                validator.operator_address
            ))
            .into());
        }
        self.bond_validator(ctx, validator)?;
        Ok(())
    }
}
