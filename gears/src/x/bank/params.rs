use database::DB;
use serde::{Deserialize, Serialize};

use crate::types::Context;

// NOTE: The send_enabled field of the bank params is hard coded to the empty list for now
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Params {
    pub default_send_enabled: bool,
}

const KEY_SEND_ENABLED: [u8; 11] = [083, 101, 110, 100, 069, 110, 097, 098, 108, 101, 100]; // "SendEnabled"
const KEY_DEFAULT_SEND_ENABLED: [u8; 18] = [
    068, 101, 102, 097, 117, 108, 116, 083, 101, 110, 100, 069, 110, 097, 098, 108, 101, 100,
]; // "DefaultSendEnabled"

const SUBSPACE_NAME: &str = "bank/";

pub const _DEFAULT_PARAMS: Params = Params {
    default_send_enabled: true,
};

impl Params {
    pub fn get<T: DB>(ctx: &Context<T>) -> Params {
        let store = ctx.get_kv_store(crate::store::Store::Params);
        let store = store.get_immutable_prefix_store(SUBSPACE_NAME.into());

        let default_send_enabled: bool = String::from_utf8(
            store
                .get(&KEY_DEFAULT_SEND_ENABLED)
                .expect("key should be set in kv store")
                .clone(),
        )
        .expect("should be valid utf-8")
        .parse()
        .expect("should be valid bool");

        Params {
            default_send_enabled,
        }
    }

    pub fn set<T: DB>(ctx: &mut Context<T>, params: Params) {
        let store = ctx.get_mutable_kv_store(crate::store::Store::Params);
        let mut store = store.get_mutable_prefix_store(SUBSPACE_NAME.into());

        store.set(
            KEY_DEFAULT_SEND_ENABLED.into(),
            params.default_send_enabled.to_string().into(),
        );

        // The send_enabled field is hard coded to the empty list for now
        store.set(KEY_SEND_ENABLED.into(), "[]".to_string().into());

        return;
    }
}
