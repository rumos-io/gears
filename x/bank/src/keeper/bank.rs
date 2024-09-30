use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M> + Send + Sync + 'static,
        M: Module,
    > BankKeeper<SK, M> for Keeper<SK, PSK, AK, M>
{
    fn send_coins_from_account_to_module<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        from_address: AccAddress,
        to_module: &M,
        amount: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        self.auth_keeper
            .check_create_new_module_account(ctx, to_module)?;

        let msg = MsgSend {
            from_address,
            to_address: to_module.get_address(),
            amount,
        };

        self.send_coins(ctx, msg)?;

        Ok(())
    }

    fn denom_metadata<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        base: &Denom,
    ) -> Result<Option<Metadata>, GasStoreErrors> {
        let bank_store = ctx.kv_store(&self.store_key);
        let denom_metadata_store = bank_store.prefix_store(denom_metadata_key(base.to_string()));

        Ok(denom_metadata_store
            .get(&base.to_string().into_bytes())?
            .map(|metadata| {
                Metadata::decode::<&[u8]>(&metadata)
                    .ok()
                    .unwrap_or_corrupt()
            }))
    }

    fn coins_burn<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        module: &M,
        deposit: &UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        let module_acc_addr = module.get_address();

        let account = self
            .auth_keeper
            .get_account(ctx, &module_acc_addr)?
            .ok_or(AccountNotFound::new(module_acc_addr.to_string()))?;

        match account.has_permissions("burner") {
            true => Ok(()),
            false => Err(BankKeeperError::AccountPermission),
        }?;

        self.sub_unlocked_coins(ctx, &module_acc_addr, deposit)?;

        for coin in deposit.inner() {
            let supply = self.supply(ctx, &coin.denom)?; // TODO: HOW TO HANDLE OPTION::NONE
            if let Some(mut supply) = supply {
                supply.amount.sub_assign(coin.amount);
                self.set_supply(ctx, supply)?;
            }
        }

        ctx.push_event(Event::new(
            "burn",
            vec![
                EventAttribute::new(
                    "burner".as_bytes().to_owned().into(),
                    account.get_address().as_ref().to_owned().into(),
                    false,
                ),
                EventAttribute::new(
                    "amount".as_bytes().to_owned().into(),
                    format!("{deposit:?}").into(),
                    false,
                ),
            ],
        ));

        Ok(())
    }

    fn send_coins_from_module_to_account<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        address: &AccAddress,
        module: &M,
        amount: UnsignedCoins,
    ) -> Result<(), BankKeeperError> {
        let module_address = module.get_address();

        // TODO: what is blocked account and how to handle it https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/bank/keeper/keeper.go#L316-L318

        self.send_coins(
            ctx,
            MsgSend {
                from_address: module_address,
                to_address: address.clone(),
                amount,
            },
        )
    }
}
