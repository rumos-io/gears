use std::collections::HashSet;

use database::Database;
use gears::{
    types::context::context::Context,
    x::params::{Keeper, ParamsSubspaceKey},
};
use proto_messages::cosmos::ibc::types::core::host::identifiers::ClientType;
use serde::{Deserialize, Serialize};
use store::StoreKey;

pub const CLIENT_PARAMS_KEY: &str = "clientParams";
pub const NEXT_CLIENT_SEQUENCE: &str = "nextClientSequence";
const ALLOW_ALL_CLIENTS: &str = "*";

#[derive(Debug, Clone)]
pub struct AbciParamsKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    pub params_keeper: Keeper<SK, PSK>,
    pub params_subspace_key: PSK,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("key should be set in kv store")]
pub struct ParamsError;

impl<SK: StoreKey, PSK: ParamsSubspaceKey> AbciParamsKeeper<SK, PSK> {
    pub fn get<DB: Database>(
        &self,
        ctx: &Context<'_, '_, DB, SK>,
        key: &impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, ParamsError> {
        let value = self
            .params_keeper
            .get_raw_subspace(ctx, &self.params_subspace_key)
            .get(key.as_ref())
            .ok_or(ParamsError)?;

        Ok(value)
    }

    pub fn set<DB: Database>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        key: impl IntoIterator<Item = u8>,
        value: impl IntoIterator<Item = u8>,
    ) {
        self.params_keeper
            .get_mutable_raw_subspace(ctx, &self.params_subspace_key)
            .set(key.into_iter().collect(), value.into_iter().collect());
    }
}

#[derive(Clone, PartialEq, prost::Message, Serialize, Deserialize)]
pub struct RawParams {
    #[prost(message, repeated, tag = "1")]
    allowed_clients: Vec<String>,
}

impl From<Params> for RawParams {
    fn from(value: Params) -> Self {
        Self {
            allowed_clients: value.allowed_clients.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    allowed_clients: HashSet<String>,
}

impl From<RawParams> for Params {
    fn from(value: RawParams) -> Self {
        Self {
            allowed_clients: value.allowed_clients.into_iter().collect(),
        }
    }
}

impl Params {
    pub fn is_client_allowed(&self, client_type: &ClientType) -> bool {
        if client_type.as_str().trim().is_empty() {
            false
        } else if self.allowed_clients.len() == 1
            && self.allowed_clients.contains(ALLOW_ALL_CLIENTS)
        {
            true
        } else {
            self.allowed_clients.contains(client_type.as_str())
        }
    }
}
