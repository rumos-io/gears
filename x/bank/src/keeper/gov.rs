use super::*;

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M> + Send + Sync + 'static,
        M: Module + strum::IntoEnumIterator,
    > GovernanceBankKeeper<SK, M> for Keeper<SK, PSK, AK, M>
{
    fn balance<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
        address: &AccAddress,
        denom: &Denom,
    ) -> Result<UnsignedCoin, GasStoreErrors> {
        let store = ctx
            .kv_store(&self.store_key)
            .prefix_store(account_key(address));

        let coin_bytes = store.get(denom.as_str().as_bytes())?;
        let coin = if let Some(coin_bytes) = coin_bytes {
            UnsignedCoin {
                denom: denom.to_owned(),
                amount: Uint256::from_str(&String::from_utf8_lossy(&coin_bytes))
                    .ok()
                    .unwrap_or_corrupt(),
            }
        } else {
            UnsignedCoin {
                denom: denom.to_owned(),
                amount: Uint256::zero(),
            }
        };

        Ok(coin)
    }
}
