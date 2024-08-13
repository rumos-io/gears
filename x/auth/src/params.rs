use std::collections::{HashMap, HashSet};

use gears::application::keepers::params::ParamsKeeper;

use gears::core::serializers::serialize_number_to_string;
use gears::params::{ParamKind, ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey};

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

impl From<ibc_proto::cosmos::auth::v1beta1::Params> for AuthsParams {
    fn from(value: ibc_proto::cosmos::auth::v1beta1::Params) -> Self {
        Self {
            max_memo_characters: value.max_memo_characters,
            tx_sig_limit: value.tx_sig_limit,
            tx_size_cost_per_byte: value.tx_size_cost_per_byte,
            sig_verify_cost_ed25519: value.sig_verify_cost_ed25519,
            sig_verify_cost_secp256k1: value.sig_verify_cost_secp256k1,
        }
    }
}

impl From<AuthsParams> for ibc_proto::cosmos::auth::v1beta1::Params {
    fn from(value: AuthsParams) -> Self {
        Self {
            max_memo_characters: value.max_memo_characters,
            tx_sig_limit: value.tx_sig_limit,
            tx_size_cost_per_byte: value.tx_size_cost_per_byte,
            sig_verify_cost_ed25519: value.sig_verify_cost_ed25519,
            sig_verify_cost_secp256k1: value.sig_verify_cost_secp256k1,
        }
    }
}

impl Default for AuthsParams {
    fn default() -> Self {
        DEFAULT_PARAMS.clone()
    }
}

impl ParamsSerialize for AuthsParams {
    fn keys() -> HashSet<&'static str> {
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

impl<PSK: ParamsSubspaceKey> ParamsKeeper<PSK> for AuthParamsKeeper<PSK> {
    type Param = AuthsParams;

    fn psk(&self) -> &PSK {
        &self.params_subspace_key
    }

    fn validate(key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> bool {
        match String::from_utf8_lossy(key.as_ref()).as_ref() {
            KEY_MAX_MEMO_CHARACTERS => ParamKind::U64
                .parse_param(value.as_ref().to_vec())
                .unsigned_64()
                .is_some(),
            KEY_TX_SIG_LIMIT => ParamKind::U64
                .parse_param(value.as_ref().to_vec())
                .unsigned_64()
                .is_some(),
            KEY_TX_SIZE_COST_PER_BYTE => ParamKind::U64
                .parse_param(value.as_ref().to_vec())
                .unsigned_64()
                .is_some(),
            KEY_SIG_VERIFY_COST_ED25519 => ParamKind::U64
                .parse_param(value.as_ref().to_vec())
                .unsigned_64()
                .is_some(),
            KEY_SIG_VERIFY_COST_SECP256K1 => ParamKind::U64
                .parse_param(value.as_ref().to_vec())
                .unsigned_64()
                .is_some(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {

    use gears::{
        derive::{ParamsKeys, StoreKeys},
        store::{bank::multi::ApplicationMultiBank, database::MemDB},
        utils::node::build_init_ctx,
    };

    use super::*;

    #[test]
    fn app_hash() {
        let keeper = AuthParamsKeeper {
            params_subspace_key: SubspaceKey::Auth,
        };

        let mut multi_store = ApplicationMultiBank::<_, SubspaceKey>::new(MemDB::new());

        let before_hash = multi_store.head_commit_hash();

        let mut ctx = build_init_ctx(&mut multi_store);

        keeper.set(&mut ctx, DEFAULT_PARAMS.clone());

        multi_store.commit();
        let after_hash = multi_store.head_commit_hash();

        assert_ne!(before_hash, after_hash);

        let expected_hash = [
            141, 88, 216, 237, 121, 214, 45, 53, 129, 175, 175, 125, 58, 187, 150, 212, 167, 90,
            83, 33, 242, 181, 88, 5, 50, 204, 98, 57, 27, 186, 208, 220,
        ];

        assert_eq!(expected_hash, after_hash);
    }

    #[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, ParamsKeys, StoreKeys)]
    #[skey(params = Params)]
    enum SubspaceKey {
        #[skey(store_str = "auth")]
        #[pkey(prefix_str = "auth")]
        Auth,
        #[skey(store_str = "param")]
        #[pkey(prefix_str = "params")]
        Params,
    }
}
