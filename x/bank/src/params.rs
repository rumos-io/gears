use std::collections::HashMap;

use gears::context::{
    InfallibleContext, InfallibleContextMut, QueryableContext, TransactionalContext,
};
use gears::params::{
    gas, infallible_subspace, infallible_subspace_mut, ParamKind, ParamsDeserialize,
    ParamsSerialize, ParamsSubspaceKey,
};
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::types::store::gas::errors::GasStoreErrors;
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

impl ParamsSerialize for BankParams {
    fn keys() -> HashMap<&'static str, ParamKind> {
        [
            (KEY_SEND_ENABLED, ParamKind::Bool),
            (KEY_DEFAULT_SEND_ENABLED, ParamKind::Bytes),
        ]
        .into_iter()
        .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut hash_map = Vec::with_capacity(2);

        hash_map.push((
            KEY_DEFAULT_SEND_ENABLED,
            self.default_send_enabled.to_string().into_bytes(),
        ));

        // The send_enabled field is hard coded to the empty list for now
        hash_map.push((KEY_SEND_ENABLED, "[]".as_bytes().to_vec()));

        hash_map
    }
}

impl ParamsDeserialize for BankParams {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            default_send_enabled: ParamKind::Bool
                .parse_param(fields.remove(KEY_DEFAULT_SEND_ENABLED).unwrap())
                .boolean()
                .unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BankParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> BankParamsKeeper<PSK> {
    pub fn get<DB: Database, SK: StoreKey, CTX: InfallibleContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> BankParams {
        let store = infallible_subspace(ctx, &self.params_subspace_key);

        store.params().unwrap_or(DEFAULT_PARAMS.clone())
    }

    pub fn set<DB: Database, SK: StoreKey, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: BankParams,
    ) {
        let mut store = infallible_subspace_mut(ctx, &self.params_subspace_key);

        store.params_set(&params)
    }

    pub fn try_get<DB: Database, SK: StoreKey, CTX: QueryableContext<DB, SK>>(
        &self,
        ctx: &CTX,
    ) -> Result<BankParams, GasStoreErrors> {
        let store = gas::subspace(ctx, &self.params_subspace_key);

        Ok(store.params()?.unwrap_or(DEFAULT_PARAMS.clone()))
    }

    pub fn try_set<DB: Database, SK: StoreKey, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: BankParams,
    ) -> Result<(), GasStoreErrors> {
        let mut store = gas::subspace_mut(ctx, &self.params_subspace_key);

        store.params_set(&params)?;

        Ok(())
    }
}
