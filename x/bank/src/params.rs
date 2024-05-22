use std::collections::{HashMap, HashSet};

use gears::params::{
    parse_primitive_unwrap, subspace, subspace_mut, Params, ParamsDeserialize, ParamsSubspaceKey,
};
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::types::context::{QueryableContext, TransactionalContext};
use serde::{Deserialize, Serialize};

const KEY_SEND_ENABLED: &str = "SendEnabled";
const KEY_DEFAULT_SEND_ENABLED: &str = "DefaultSendEnabled";

// NOTE: The send_enabled field of the bank params is hard coded to the empty list for now
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BankParams {
    pub default_send_enabled: bool,
}

pub const DEFAULT_PARAMS: BankParams = BankParams {
    default_send_enabled: true,
};

impl Params for BankParams {
    fn keys() -> HashSet<&'static str> {
        [KEY_SEND_ENABLED, KEY_DEFAULT_SEND_ENABLED]
            .into_iter()
            .collect()
    }

    fn serialize(&self) -> HashMap<&'static str, Vec<u8>> {
        let mut hash_map = HashMap::with_capacity(2);

        hash_map.insert(
            KEY_DEFAULT_SEND_ENABLED,
            self.default_send_enabled.to_string().into_bytes(),
        );

        // The send_enabled field is hard coded to the empty list for now
        hash_map.insert(KEY_SEND_ENABLED, "[]".as_bytes().to_vec());

        hash_map
    }
}

impl ParamsDeserialize for BankParams {
    fn deserialize(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            default_send_enabled: parse_primitive_unwrap(fields.remove(KEY_DEFAULT_SEND_ENABLED)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BankParamsKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    pub store_key: SK,
    pub params_subspace_key: PSK,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> BankParamsKeeper<SK, PSK> {
    pub fn get<DB: Database, CTX: QueryableContext<DB, SK>>(&self, ctx: &CTX) -> BankParams {
        let store = subspace(ctx, &self.store_key, &self.params_subspace_key);

        store.params().expect("Required to be set")
    }

    pub fn set<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: BankParams,
    ) {
        let mut store = subspace_mut(ctx, &self.store_key, &self.params_subspace_key);

        store.params_set(&params)
    }
}
