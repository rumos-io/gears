use gears::{
    context::{InfallibleContext, InfallibleContextMut, QueryableContext, TransactionalContext},
    core::serializers::serialize_number_to_string,
    params::{
        gas, infallible_subspace, infallible_subspace_mut, ParamKind, ParamsDeserialize,
        ParamsSerialize, ParamsSubspaceKey,
    },
    store::{database::Database, StoreKey},
    types::{decimal256::Decimal256, store::gas::errors::GasStoreErrors},
};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

const KEY_SIGNED_BLOCKS_WINDOW: &str = "SignedBlocksWindow";
const KEY_MIN_SIGNED_PER_WINDOW: &str = "MinSignedPerWindow";
const KEY_DOWNTIME_JAIL_DURATION: &str = "DowntimeJailDuration";
const KEY_SLASH_FRACTION_DOUBLE_SIGN: &str = "SlashFractionDoubleSign";
const KEY_SLASH_FRACTION_DOWNTIME: &str = "SlashFractionDowntime";

/// SlashingParams represents the parameters used for by the slashing module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlashingParams {
    #[serde(serialize_with = "serialize_number_to_string")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub signed_blocks_window: i64,
    pub min_signed_per_window: Decimal256,
    #[serde(serialize_with = "serialize_number_to_string")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub downtime_jail_duration: i64,
    pub slash_fraction_double_sign: Decimal256,
    pub slash_fraction_downtime: Decimal256,
}

impl ParamsSerialize for SlashingParams {
    fn keys() -> HashSet<&'static str> {
        [
            KEY_SIGNED_BLOCKS_WINDOW,
            KEY_MIN_SIGNED_PER_WINDOW,
            KEY_DOWNTIME_JAIL_DURATION,
            KEY_SLASH_FRACTION_DOUBLE_SIGN,
            KEY_SLASH_FRACTION_DOWNTIME,
        ]
        .into_iter()
        .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut raws = Vec::with_capacity(5);
        raws.push((
            KEY_SIGNED_BLOCKS_WINDOW,
            // TODO: check during integration of genesis
            format!("\"{}\"", self.signed_blocks_window).into_bytes(),
        ));
        raws.push((
            // TODO: check during integration of genesis
            KEY_MIN_SIGNED_PER_WINDOW,
            self.min_signed_per_window.to_string().into_bytes(),
        ));
        raws.push((
            KEY_DOWNTIME_JAIL_DURATION,
            // TODO: check during integration of genesis
            format!("\"{}\"", self.downtime_jail_duration).into_bytes(),
        ));
        raws.push((
            KEY_SLASH_FRACTION_DOUBLE_SIGN,
            self.slash_fraction_double_sign.to_string().into_bytes(),
        ));
        raws.push((
            KEY_SLASH_FRACTION_DOWNTIME,
            self.slash_fraction_downtime.to_string().into_bytes(),
        ));
        raws
    }
}

impl ParamsDeserialize for SlashingParams {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            signed_blocks_window: ParamKind::I64
                .parse_param(fields.remove(KEY_SIGNED_BLOCKS_WINDOW).unwrap())
                .signed_64()
                .unwrap(),
            min_signed_per_window: Decimal256::from_str(
                &String::from_utf8(
                    ParamKind::Bytes
                        .parse_param(fields.remove(KEY_MIN_SIGNED_PER_WINDOW).unwrap())
                        .bytes()
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
            downtime_jail_duration: ParamKind::I64
                .parse_param(fields.remove(KEY_DOWNTIME_JAIL_DURATION).unwrap())
                .signed_64()
                .unwrap(),
            slash_fraction_double_sign: Decimal256::from_str(
                &String::from_utf8(
                    ParamKind::Bytes
                        .parse_param(fields.remove(KEY_SLASH_FRACTION_DOUBLE_SIGN).unwrap())
                        .bytes()
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
            slash_fraction_downtime: Decimal256::from_str(
                &String::from_utf8(
                    ParamKind::Bytes
                        .parse_param(fields.remove(KEY_SLASH_FRACTION_DOWNTIME).unwrap())
                        .bytes()
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        }
    }
}

impl Default for SlashingParams {
    fn default() -> Self {
        // TODO: check defaults, especially with division
        Self {
            signed_blocks_window: 100,
            min_signed_per_window: Decimal256::from_atomics(5u64, 1).unwrap(),
            downtime_jail_duration: 60 * 10 * 1_000_000_000,
            slash_fraction_double_sign: Decimal256::one()
                .checked_div(Decimal256::from_atomics(20u64, 0).unwrap())
                .unwrap(),
            slash_fraction_downtime: Decimal256::one()
                .checked_div(Decimal256::from_atomics(100u64, 0).unwrap())
                .unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlashingParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> SlashingParamsKeeper<PSK> {
    pub fn get<DB: Database, SK: StoreKey, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> SlashingParams {
        let store = infallible_subspace(ctx, &self.params_subspace_key);
        store.params().unwrap_or(SlashingParams::default())
    }

    pub fn try_get<DB: Database, SK: StoreKey, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<SlashingParams, GasStoreErrors> {
        let store = gas::subspace(ctx, &self.params_subspace_key);

        Ok(store.params()?.unwrap_or(SlashingParams::default()))
    }

    pub fn set<DB: Database, SK: StoreKey, KV: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut KV,
        params: SlashingParams,
    ) {
        let mut store = infallible_subspace_mut(ctx, &self.params_subspace_key);
        store.params_set(&params)
    }

    pub fn try_set<DB: Database, SK: StoreKey, KV: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut KV,
        params: SlashingParams,
    ) -> Result<(), GasStoreErrors> {
        let mut store = gas::subspace_mut(ctx, &self.params_subspace_key);
        store.params_set(&params)
    }
}
