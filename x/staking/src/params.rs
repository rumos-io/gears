use gears::core::base::coin::Coin;
use gears::store::StoreKey;
use gears::{
    params::ParamsSubspaceKey,
    store::{
        QueryableMultiKVStore, TransactionalMultiKVStore,
        database::{Database, PrefixDB},
        ReadPrefixStore, WritePrefixStore,
    },
    types::{
        denom::Denom,
    },
};
use serde::{de::Error, Deserialize, Serialize};

const PARAMS_KEY: [u8; 6] = [112, 97, 114, 97, 109, 115]; // "params"

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Params {
    pub unbonding_time: std::time::Duration,
    pub max_validators: u32,
    pub max_entries: u32,
    pub historical_entries: u32,
    pub bond_denom: Denom,
    pub min_commission_rate: Coin,
}

impl Default for Params {
    fn default() -> Self {
        // TODO: remove unwrap, maybe propose default value
        let bond_denom = Denom::try_from("uatom".to_string()).unwrap();
        Params {
            // 3 weeks
            unbonding_time: std::time::Duration::from_secs(60 * 60 * 24 * 7 * 3),
            max_validators: 100,
            max_entries: 7,
            bond_denom: bond_denom.clone(),
            historical_entries: 0,
            min_commission_rate: Coin::default(),
        }
    }
}

impl Params {
    pub fn validate(&self) -> Result<(), String> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct StakingParamsKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    pub params_keeper: gears::params::Keeper<SK, PSK>,
    pub params_subspace_key: PSK,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> StakingParamsKeeper<SK, PSK> {
    pub fn get<DB: Database, CTX: QueryableMultiKVStore<PrefixDB<DB>, SK>>(
        &self,
        ctx: &mut CTX,
    ) -> anyhow::Result<Params> {
        let store = self
            .params_keeper
            .raw_subspace(ctx, &self.params_subspace_key);
        if let Some(raw_params) = store.get(PARAMS_KEY.as_ref()) {
            Ok(serde_json::from_slice(&raw_params)?)
        } else {
            Err(serde_json::Error::custom("Cannot find data to convert".to_string()).into())
        }
    }

    pub fn set<DB: Database, CTX: TransactionalMultiKVStore<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: Params,
    ) -> anyhow::Result<()> {
        let mut store = self
            .params_keeper
            .raw_subspace_mut(ctx, &self.params_subspace_key);
        store.set(PARAMS_KEY, serde_json::to_vec(&params)?);
        Ok(())
    }
}
