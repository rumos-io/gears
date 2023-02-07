use crate::{
    store::{KVStore, StoreKey},
    types::Context,
};

#[derive(Debug, Clone)]
pub struct Params {
    pub max_memo_characters: u64,
    pub tx_sig_limit: u64,
    pub tx_size_cost_per_byte: u64,
    pub sig_verify_cost_ed25519: u64,
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

pub const DEFAULT_PARAMS: Params = Params {
    max_memo_characters: 256,
    tx_sig_limit: 7,
    tx_size_cost_per_byte: 10,
    sig_verify_cost_ed25519: 590,
    sig_verify_cost_secp256k1: 1000,
};

impl Params {
    pub fn get(ctx: &Context) -> Params {
        let store = ctx.get_kv_store(crate::store::StoreKey::AuthParams);

        let max_memo_characters: u64 = String::from_utf8(
            store
                .get(&KEY_MAX_MEMO_CHARACTERS)
                .expect("key should be set in kv store")
                .clone(),
        )
        .expect("should be valid utf-8")
        .parse()
        .expect("should be valid u64");

        let tx_sig_limit: u64 = String::from_utf8(
            store
                .get(&KEY_TX_SIG_LIMIT)
                .expect("key should be set in kv store")
                .clone(),
        )
        .expect("should be valid utf-8")
        .parse()
        .expect("should be valid u64");

        let tx_size_cost_per_byte: u64 = String::from_utf8(
            store
                .get(&KEY_TX_SIZE_COST_PER_BYTE)
                .expect("key should be set in kv store")
                .clone(),
        )
        .expect("should be valid utf-8")
        .parse()
        .expect("should be valid u64");

        let sig_verify_cost_ed25519: u64 = String::from_utf8(
            store
                .get(&KEY_SIG_VERIFY_COST_ED25519)
                .expect("key should be set in kv store")
                .clone(),
        )
        .expect("should be valid utf-8")
        .parse()
        .expect("should be valid u64");

        let sig_verify_cost_secp256k1: u64 = String::from_utf8(
            store
                .get(&KEY_SIG_VERIFY_COST_SECP256K1)
                .expect("key should be set in kv store")
                .clone(),
        )
        .expect("should be valid utf-8")
        .parse()
        .expect("should be valid u64");

        Params {
            max_memo_characters,
            tx_sig_limit,
            tx_size_cost_per_byte,
            sig_verify_cost_ed25519,
            sig_verify_cost_secp256k1,
        }
    }

    pub fn set(ctx: &mut Context, params: Params) {
        let store = ctx.get_mutable_kv_store(crate::store::StoreKey::AuthParams);

        store.set(
            KEY_MAX_MEMO_CHARACTERS.into(),
            params.max_memo_characters.to_string().into(),
        );

        store.set(
            KEY_TX_SIG_LIMIT.into(),
            params.tx_sig_limit.to_string().into(),
        );

        store.set(
            KEY_TX_SIZE_COST_PER_BYTE.into(),
            params.tx_size_cost_per_byte.to_string().into(),
        );

        store.set(
            KEY_SIG_VERIFY_COST_ED25519.into(),
            params.sig_verify_cost_ed25519.to_string().into(),
        );

        store.set(
            KEY_SIG_VERIFY_COST_SECP256K1.into(),
            params.sig_verify_cost_secp256k1.to_string().into(),
        );

        return;
    }
}
