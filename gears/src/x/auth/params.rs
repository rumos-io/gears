use database::DB;
use proto_messages::utils::serialize_number_to_string;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;

use crate::{store::ImmutablePrefixStore, types::Context};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Params {
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

const KEY_MAX_MEMO_CHARACTERS: [u8; 17] = [
    077, 097, 120, 077, 101, 109, 111, 067, 104, 097, 114, 097, 099, 116, 101, 114, 115,
]; // "MaxMemoCharacters"
const KEY_TX_SIG_LIMIT: [u8; 10] = [084, 120, 083, 105, 103, 076, 105, 109, 105, 116]; // "TxSigLimit"
const KEY_TX_SIZE_COST_PER_BYTE: [u8; 17] = [
    084, 120, 083, 105, 122, 101, 067, 111, 115, 116, 080, 101, 114, 066, 121, 116, 101,
]; // "TxSizeCostPerByte"
const KEY_SIG_VERIFY_COST_ED25519: [u8; 20] = [
    083, 105, 103, 086, 101, 114, 105, 102, 121, 067, 111, 115, 116, 069, 068, 050, 053, 053, 049,
    057,
]; // "SigVerifyCostED25519"
const KEY_SIG_VERIFY_COST_SECP256K1: [u8; 22] = [
    083, 105, 103, 086, 101, 114, 105, 102, 121, 067, 111, 115, 116, 083, 101, 099, 112, 050, 053,
    054, 107, 049,
]; // "SigVerifyCostSecp256k1"

const SUBSPACE_NAME: &str = "auth/";

// pub const DEFAULT_PARAMS: Params = Params {
//     max_memo_characters: 256,
//     tx_sig_limit: 7,
//     tx_size_cost_per_byte: 10,
//     sig_verify_cost_ed25519: 590,
//     sig_verify_cost_secp256k1: 1000,
// };

impl Params {
    fn parse_param(value: Vec<u8>) -> u64 {
        String::from_utf8(value)
            .expect("should be valid utf-8")
            .strip_suffix("\"")
            .expect("should have suffix")
            .strip_prefix("\"")
            .expect("should have prefix")
            .parse()
            .expect("should be valid u64")
    }

    fn get_raw_param<T: DB>(key: &[u8], store: &ImmutablePrefixStore<T>) -> Vec<u8> {
        store
            .get(key)
            .expect("key should be set in kv store")
            .clone()
    }

    pub fn get<T: DB>(ctx: &Context<T>) -> Params {
        let store = ctx.get_kv_store(crate::store::Store::Params);
        let store = store.get_immutable_prefix_store(SUBSPACE_NAME.into());

        let raw = Params::get_raw_param(&KEY_MAX_MEMO_CHARACTERS, &store);
        let max_memo_characters = Params::parse_param(raw);

        let raw = Params::get_raw_param(&KEY_TX_SIG_LIMIT, &store);
        let tx_sig_limit = Params::parse_param(raw);

        let raw = Params::get_raw_param(&KEY_TX_SIZE_COST_PER_BYTE, &store);
        let tx_size_cost_per_byte = Params::parse_param(raw);

        let raw = Params::get_raw_param(&KEY_SIG_VERIFY_COST_ED25519, &store);
        let sig_verify_cost_ed25519 = Params::parse_param(raw);

        let raw = Params::get_raw_param(&KEY_SIG_VERIFY_COST_SECP256K1, &store);
        let sig_verify_cost_secp256k1 = Params::parse_param(raw);

        Params {
            max_memo_characters,
            tx_sig_limit,
            tx_size_cost_per_byte,
            sig_verify_cost_ed25519,
            sig_verify_cost_secp256k1,
        }
    }

    pub fn set<T: DB>(ctx: &mut Context<T>, params: Params) {
        let store = ctx.get_mutable_kv_store(crate::store::Store::Params);
        let mut store = store.get_mutable_prefix_store(SUBSPACE_NAME.into());

        store.set(
            KEY_MAX_MEMO_CHARACTERS.into(),
            format!("\"{}\"", params.max_memo_characters.to_string()).into(),
        );

        store.set(
            KEY_TX_SIG_LIMIT.into(),
            format!("\"{}\"", params.tx_sig_limit.to_string()).into(),
        );

        store.set(
            KEY_TX_SIZE_COST_PER_BYTE.into(),
            format!("\"{}\"", params.tx_size_cost_per_byte.to_string()).into(),
        );

        store.set(
            KEY_SIG_VERIFY_COST_ED25519.into(),
            format!("\"{}\"", params.sig_verify_cost_ed25519.to_string()).into(),
        );

        store.set(
            KEY_SIG_VERIFY_COST_SECP256K1.into(),
            format!("\"{}\"", params.sig_verify_cost_secp256k1.to_string()).into(),
        );

        return;
    }
}
