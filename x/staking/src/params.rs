use gears::{
    context::{InfallibleContext, InfallibleContextMut, QueryableContext, TransactionalContext},
    core::base::coin::Coin,
    params::{ParamKind, ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey},
    store::{database::Database, StoreKey},
    tendermint::types::time::Duration,
    types::{denom::Denom, store::errors::StoreErrors},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const KEY_UNBONDING_TIME: &str = "UnbondingTime";
const KEY_MAX_VALIDATORS: &str = "MaxValidators";
const KEY_MAX_ENTRIES: &str = "MaxEntries";
const KEY_HISTORICAL_ENTRIES: &str = "HistoricalEntries";
const KEY_BOND_DENOM: &str = "BondDenom";
const KEY_MIN_COMMISSION_RATE: &str = "MinCommissionRate";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Params {
    pub unbonding_time: Duration,
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
            unbonding_time: Duration {
                seconds: 60 * 60 * 24 * 7 * 3,
                nanos: 0,
            },
            max_validators: 100,
            max_entries: 7,
            bond_denom: bond_denom.clone(),
            historical_entries: 0,
            min_commission_rate: Coin::default(),
        }
    }
}

impl ParamsSerialize for Params {
    fn keys() -> HashMap<&'static str, ParamKind> {
        [
            (KEY_UNBONDING_TIME, ParamKind::Bytes),
            (KEY_MAX_VALIDATORS, ParamKind::U32),
            (KEY_MAX_ENTRIES, ParamKind::U32),
            (KEY_HISTORICAL_ENTRIES, ParamKind::U32),
            (KEY_BOND_DENOM, ParamKind::String),
            (KEY_MIN_COMMISSION_RATE, ParamKind::Bytes),
        ]
        .into_iter()
        .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        vec![
            // TODO: remove unwrap
            (
                KEY_UNBONDING_TIME,
                serde_json::to_vec(&self.unbonding_time).unwrap(),
            ),
            (
                KEY_MAX_VALIDATORS,
                self.max_validators.to_string().as_bytes().to_vec(),
            ),
            (
                KEY_MAX_ENTRIES,
                self.max_entries.to_string().as_bytes().to_vec(),
            ),
            (
                KEY_HISTORICAL_ENTRIES,
                self.historical_entries.to_string().as_bytes().to_vec(),
            ),
            (
                KEY_BOND_DENOM,
                self.max_validators.to_string().as_bytes().to_vec(),
            ),
            // TODO: remove unwrap
            (
                KEY_MIN_COMMISSION_RATE,
                serde_json::to_vec(&self.min_commission_rate).unwrap(),
            ),
        ]
    }
}

impl ParamsDeserialize for Params {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        // TODO: check unwraps
        let unbonding_time: Duration = serde_json::from_slice(
            &ParamKind::Bytes
                .parse_param(fields.remove(KEY_UNBONDING_TIME).unwrap())
                .bytes()
                .unwrap(),
        )
        .unwrap();
        let max_validators = ParamKind::U32
            .parse_param(fields.remove(KEY_MAX_VALIDATORS).unwrap())
            .unsigned_64()
            .unwrap() as u32;
        let max_entries = ParamKind::U32
            .parse_param(fields.remove(KEY_MAX_ENTRIES).unwrap())
            .unsigned_64()
            .unwrap() as u32;
        let historical_entries = ParamKind::U32
            .parse_param(fields.remove(KEY_HISTORICAL_ENTRIES).unwrap())
            .unsigned_64()
            .unwrap() as u32;
        let bond_denom = ParamKind::String
            .parse_param(fields.remove(KEY_BOND_DENOM).unwrap())
            .string()
            .unwrap()
            .try_into()
            .unwrap();
        let min_commission_rate: Coin = serde_json::from_slice(
            &ParamKind::Bytes
                .parse_param(fields.remove(KEY_MIN_COMMISSION_RATE).unwrap())
                .bytes()
                .unwrap(),
        )
        .unwrap();

        Params {
            unbonding_time,
            max_validators,
            max_entries,
            bond_denom,
            historical_entries,
            min_commission_rate,
        }
    }
}

impl Params {
    pub fn validate(&self) -> Result<(), String> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct StakingParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> StakingParamsKeeper<PSK> {
    pub fn get<DB: Database, SK: StoreKey, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Params {
        let store = gears::params::infallible_subspace(ctx, &self.params_subspace_key);
        store.params().expect("params should be stored in database")
    }

    pub fn set<DB: Database, SK: StoreKey, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: Params,
    ) {
        let mut store = gears::params::infallible_subspace_mut(ctx, &self.params_subspace_key);
        store.params_set(&params);
    }

    pub fn try_get<DB: Database, SK: StoreKey, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<Params, StoreErrors> {
        let store = gears::params::gas::subspace(ctx, &self.params_subspace_key);
        Ok(store
            .params()?
            .expect("params should be stored in database"))
    }

    pub fn try_set<DB: Database, SK: StoreKey, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: Params,
    ) -> Result<(), StoreErrors> {
        let mut store = gears::params::gas::subspace_mut(ctx, &self.params_subspace_key);
        store.params_set(&params)
    }
}
