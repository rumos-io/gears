use std::collections::HashMap;

use gears::context::{
    InfallibleContext, InfallibleContextMut, QueryableContext, TransactionalContext,
};
use gears::core::serializers::serialize_number_to_string;
use gears::params::{
    gas, subspace, subspace_mut, ParamKind, ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey,
};
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::types::store::errors::StoreErrors;
use gears::x::keepers::auth::AuthParams;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;

const KEY_MAX_MEMO_CHARACTERS: &str = "MaxMemoCharacters";
const KEY_TX_SIG_LIMIT: &str = "TxSigLimit";
const KEY_TX_SIZE_COST_PER_BYTE: &str = "TxSizeCostPerByte";
const KEY_SIG_VERIFY_COST_ED25519: &str = "SigVerifyCostED25519";
const KEY_SIG_VERIFY_COST_SECP256K1: &str = "SigVerifyCostSecp256k1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthsParams {
    #[serde(serialize_with = "serialize_number_to_string")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub max_memo_characters: u64,
    #[serde(serialize_with = "serialize_number_to_string")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub tx_sig_limit: u64,
    #[serde(serialize_with = "serialize_number_to_string")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub tx_size_cost_per_byte: u64,
    #[serde(serialize_with = "serialize_number_to_string")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub sig_verify_cost_ed25519: u64,
    #[serde(serialize_with = "serialize_number_to_string")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub sig_verify_cost_secp256k1: u64,
}

impl ParamsSerialize for AuthsParams {
    fn keys() -> HashMap<&'static str, ParamKind> {
        [
            (KEY_MAX_MEMO_CHARACTERS, ParamKind::U64),
            (KEY_TX_SIG_LIMIT, ParamKind::U64),
            (KEY_TX_SIZE_COST_PER_BYTE, ParamKind::U64),
            (KEY_SIG_VERIFY_COST_ED25519, ParamKind::U64),
            (KEY_SIG_VERIFY_COST_SECP256K1, ParamKind::U64),
        ]
        .into_iter()
        .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut hash_map = Vec::with_capacity(5);

        hash_map.push((
            KEY_MAX_MEMO_CHARACTERS,
            format!("\"{}\"", self.max_memo_characters).into_bytes(),
        ));

        hash_map.push((
            KEY_TX_SIG_LIMIT,
            format!("\"{}\"", self.tx_sig_limit).into_bytes(),
        ));

        hash_map.push((
            KEY_TX_SIZE_COST_PER_BYTE,
            format!("\"{}\"", self.tx_size_cost_per_byte).into_bytes(),
        ));

        hash_map.push((
            KEY_SIG_VERIFY_COST_ED25519,
            format!("\"{}\"", self.sig_verify_cost_ed25519).into_bytes(),
        ));

        hash_map.push((
            KEY_SIG_VERIFY_COST_SECP256K1,
            format!("\"{}\"", self.sig_verify_cost_secp256k1).into_bytes(),
        ));

        hash_map
    }
}

impl ParamsDeserialize for AuthsParams {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        // TODO:NOW THIS IS AWFUL
        Self {
            max_memo_characters: ParamKind::U64
                .parse_param(fields.remove(KEY_MAX_MEMO_CHARACTERS).unwrap())
                .unsigned_64()
                .unwrap(),
            tx_sig_limit: ParamKind::U64
                .parse_param(fields.remove(KEY_TX_SIG_LIMIT).unwrap())
                .unsigned_64()
                .unwrap(),
            tx_size_cost_per_byte: ParamKind::U64
                .parse_param(fields.remove(KEY_TX_SIZE_COST_PER_BYTE).unwrap())
                .unsigned_64()
                .unwrap(),
            sig_verify_cost_ed25519: ParamKind::U64
                .parse_param(fields.remove(KEY_SIG_VERIFY_COST_ED25519).unwrap())
                .unsigned_64()
                .unwrap(),
            sig_verify_cost_secp256k1: ParamKind::U64
                .parse_param(fields.remove(KEY_SIG_VERIFY_COST_SECP256K1).unwrap())
                .unsigned_64()
                .unwrap(),
        }
    }
}

impl AuthParams for AuthsParams {
    fn max_memo_characters(&self) -> u64 {
        self.max_memo_characters
    }

    fn sig_verify_cost_secp256k1(&self) -> u64 {
        self.sig_verify_cost_secp256k1
    }

    fn tx_cost_per_byte(&self) -> u64 {
        self.tx_size_cost_per_byte
    }
}

pub const DEFAULT_PARAMS: AuthsParams = AuthsParams {
    max_memo_characters: 256,
    tx_sig_limit: 7,
    tx_size_cost_per_byte: 10,
    sig_verify_cost_ed25519: 590,
    sig_verify_cost_secp256k1: 1000,
};

#[derive(Debug, Clone)]
pub struct AuthParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> AuthParamsKeeper<PSK> {
    pub fn get<DB: Database, SK: StoreKey, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> AuthsParams {
        let store = subspace(ctx, &self.params_subspace_key);

        store.params().unwrap_or(DEFAULT_PARAMS.clone())
    }

    pub fn get_with_gas<DB: Database, SK: StoreKey, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<AuthsParams, StoreErrors> {
        let store = gas::subspace(ctx, &self.params_subspace_key);

        Ok(store.params()?.unwrap_or(DEFAULT_PARAMS.clone()))
    }

    pub fn set<DB: Database, SK: StoreKey, KV: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut KV,
        params: AuthsParams,
    ) {
        let mut store = subspace_mut(ctx, &self.params_subspace_key);

        store.params_set(&params)
    }

    pub fn set_with_gas<DB: Database, SK: StoreKey, KV: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut KV,
        params: AuthsParams,
    ) -> Result<(), StoreErrors> {
        let mut store = gas::subspace_mut(ctx, &self.params_subspace_key);

        store.params_set(&params)
    }
}
