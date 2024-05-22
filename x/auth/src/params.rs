use std::collections::HashMap;

use gears::core::serializers::serialize_number_to_string;
use gears::params::{
    parse_primitive_unwrap, subspace, subspace_mut, Params, ParamsDeserialize, ParamsSubspaceKey,
};
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::types::context::{QueryableContext, TransactionalContext};
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

impl Params for AuthsParams {
    fn keys() -> std::collections::HashSet<&'static str> {
        [
            KEY_MAX_MEMO_CHARACTERS,
            KEY_TX_SIG_LIMIT,
            KEY_TX_SIZE_COST_PER_BYTE,
            KEY_SIG_VERIFY_COST_ED25519,
            KEY_SIG_VERIFY_COST_SECP256K1,
        ]
        .into_iter()
        .collect()
    }

    fn to_raw(&self) -> std::collections::HashMap<&'static str, Vec<u8>> {
        let mut hash_map = HashMap::with_capacity(5);

        hash_map.insert(
            KEY_MAX_MEMO_CHARACTERS,
            format!("\"{}\"", self.max_memo_characters).into_bytes(),
        );

        hash_map.insert(
            KEY_TX_SIG_LIMIT,
            format!("\"{}\"", self.tx_sig_limit).into_bytes(),
        );

        hash_map.insert(
            KEY_TX_SIZE_COST_PER_BYTE,
            format!("\"{}\"", self.tx_size_cost_per_byte).into_bytes(),
        );

        hash_map.insert(
            KEY_SIG_VERIFY_COST_ED25519,
            format!("\"{}\"", self.sig_verify_cost_ed25519).into_bytes(),
        );

        hash_map.insert(
            KEY_SIG_VERIFY_COST_SECP256K1,
            format!("\"{}\"", self.sig_verify_cost_secp256k1).into_bytes(),
        );

        hash_map
    }
}

impl ParamsDeserialize for AuthsParams {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            max_memo_characters: parse_primitive_unwrap(fields.remove(KEY_MAX_MEMO_CHARACTERS)),
            tx_sig_limit: parse_primitive_unwrap(fields.remove(KEY_TX_SIG_LIMIT)),
            tx_size_cost_per_byte: parse_primitive_unwrap(fields.remove(KEY_TX_SIZE_COST_PER_BYTE)),
            sig_verify_cost_ed25519: parse_primitive_unwrap(
                fields.remove(KEY_SIG_VERIFY_COST_ED25519),
            ),
            sig_verify_cost_secp256k1: parse_primitive_unwrap(
                fields.remove(KEY_SIG_VERIFY_COST_SECP256K1),
            ),
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
pub struct AuthParamsKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    pub store_key: SK,
    pub params_subspace_key: PSK,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> AuthParamsKeeper<SK, PSK> {
    pub fn get<DB: Database, CTX: QueryableContext<DB, SK>>(&self, ctx: &CTX) -> AuthsParams {
        let store = subspace(ctx, &self.store_key, &self.params_subspace_key);

        store.params().expect("Required to exists")
    }

    pub fn set<DB: Database, KV: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut KV,
        params: AuthsParams,
    ) {
        let mut store = subspace_mut(ctx, &self.store_key, &self.params_subspace_key);

        store.params_set(&params)
    }
}
