use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M> + Send + Sync + 'static,
        M: Module,
    > BalancesKeeper<SK, M> for Keeper<SK, PSK, AK, M>
{
    fn balance_all<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        addr: AccAddress,
        pagination: Option<Pagination>,
    ) -> Result<(Option<PaginationResult>, Vec<UnsignedCoin>), GasStoreErrors> {
        let bank_store = ctx.kv_store(&self.store_key);
        let prefix = create_denom_balance_prefix(addr.clone());
        let account_store = bank_store.prefix_store(prefix);

        let mut balances = vec![];

        let (p_result, iterator) = account_store.into_range(..).maybe_paginate(pagination);
        for rcoin in iterator {
            let (_, coin) = rcoin?;
            let coin: UnsignedCoin = UnsignedCoin::decode::<Bytes>(coin.into_owned().into())
                .ok()
                .unwrap_or_corrupt();
            balances.push(coin);
        }
        Ok((p_result, balances))
    }

    fn supply<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        denom: &Denom,
    ) -> Result<Option<UnsignedCoin>, GasStoreErrors> {
        let supply_store = ctx.kv_store(&self.store_key).prefix_store(SUPPLY_KEY);

        let amount_bytes = supply_store.get(denom)?;

        match amount_bytes {
            Some(bytes) => Ok(Some(UnsignedCoin {
                denom: denom.clone(),
                amount: Uint256::from_str(&String::from_utf8_lossy(&bytes))
                    .ok()
                    .unwrap_or_corrupt(),
            })),
            None => Ok(None),
        }
    }
}
