use std::collections::{HashMap, HashSet};

use database::Database;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use store_crate::StoreKey;
use tendermint::types::proto::consensus::ConsensusParams;

use crate::{
    params::{string::ParamString, subspace, subspace_mut, Params, ParamsSubspaceKey},
    types::context::{QueryableContext, TransactionalContext},
};

mod inner {
    pub use tendermint::types::proto::params::BlockParams;
    pub use tendermint::types::proto::params::EvidenceParams;
    pub use tendermint::types::proto::params::ValidatorParams;
}

const KEY_BLOCK_PARAMS: &str = "BlockParams"; //[u8; 11] = [066, 108, 111, 099, 107, 080, 097, 114, 097, 109, 115]; // "BlockParams"
const KEY_EVIDENCE_PARAMS: &str = "EvidenceParams";
// [u8; 14] = [
//     069, 118, 105, 100, 101, 110, 099, 101, 080, 097, 114, 097, 109, 115,
// ]; // "EvidenceParams"

const KEY_VALIDATOR_PARAMS: &str = "ValidatorParams";
// [u8; 15] = [
//     086, 097, 108, 105, 100, 097, 116, 111, 114, 080, 097, 114, 097, 109, 115,
// ]; // "ValidatorParams"

const _SUBSPACE_NAME: &str = "baseapp/";

const SEC_TO_NANO: i64 = 1_000_000_000;

//##################################################################################
//##################################################################################
// TODO: The cosmos sdk / tendermint uses a custom serializer/deserializer
// we've replicated the behaviour with a hacked combination of using serde_json
// and string types. Apart from being a mess, this conversion to JSON isn't
// deterministic, presumably the SDK handles this.
//##################################################################################
//##################################################################################

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct BlockParams {
    pub max_bytes: String,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub max_gas: i64,
}

impl From<ParamString> for BlockParams {
    fn from(value: ParamString) -> Self {
        let var = serde_json::from_str(&value.0);

        var.expect("Should be valid")
    }
}

impl From<inner::BlockParams> for BlockParams {
    fn from(params: inner::BlockParams) -> BlockParams {
        BlockParams {
            max_bytes: params.max_bytes.to_string(),
            max_gas: params.max_gas,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ValidatorParams {
    pub pub_key_types: Vec<String>,
}

impl From<inner::ValidatorParams> for ValidatorParams {
    fn from(params: inner::ValidatorParams) -> ValidatorParams {
        ValidatorParams {
            pub_key_types: params.pub_key_types,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EvidenceParams {
    max_age_num_blocks: String,
    max_age_duration: Option<String>,
    max_bytes: String,
}

impl From<inner::EvidenceParams> for EvidenceParams {
    fn from(params: inner::EvidenceParams) -> EvidenceParams {
        let duration = params
            .max_age_duration
            .map(|dur| dur.seconds * SEC_TO_NANO + i64::from(dur.nanos));

        EvidenceParams {
            max_age_num_blocks: params.max_age_num_blocks.to_string(),
            max_age_duration: duration.map(|val| val.to_string()),
            max_bytes: params.max_bytes.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BaseAppParamsKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    pub store_key: SK,
    pub params_subspace_key: PSK,
}

// TODO: add a macro to create this?
impl<SK: StoreKey, PSK: ParamsSubspaceKey> BaseAppParamsKeeper<SK, PSK> {
    pub fn set_consensus_params<DB: Database, CTX: TransactionalContext<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: ConsensusParams,
    ) {
        let mut store = subspace_mut(ctx, &self.store_key, &self.params_subspace_key);

        store.params_set(&params);
    }

    pub fn block_params<DB: Database, CTX: QueryableContext<DB, SK>>(
        &self,
        store: &CTX,
    ) -> Option<BlockParams> {
        let sub_store = subspace(store, &self.store_key, &self.params_subspace_key);

        sub_store.params_field::<BlockParams>(KEY_BLOCK_PARAMS)
    }
}

impl Params for ConsensusParams {
    fn keys() -> HashSet<&'static str> {
        [KEY_BLOCK_PARAMS, KEY_EVIDENCE_PARAMS, KEY_VALIDATOR_PARAMS]
            .into_iter()
            .collect()
    }

    fn to_raw(&self) -> HashMap<&'static str, ParamString> {
        let mut hash_map = HashMap::with_capacity(3);

        if let Some(params) = self.block.clone() {
            let block_params = serde_json::to_string(&BlockParams::from(params))
                .expect("conversion to json won't fail");
            hash_map.insert(KEY_BLOCK_PARAMS, block_params.into());
        }

        if let Some(params) = self.evidence.clone() {
            let evidence_params = serde_json::to_string(&EvidenceParams::from(params))
                .expect("conversion to json won't fail");
            hash_map.insert(KEY_EVIDENCE_PARAMS, evidence_params.into());
        }

        if let Some(params) = self.validator.clone() {
            let params = serde_json::to_string(&ValidatorParams::from(params))
                .expect("conversion to json won't fail");
            hash_map.insert(KEY_VALIDATOR_PARAMS, params.into());
        }

        hash_map
    }
}

#[cfg(test)]
mod tests {
    use super::EvidenceParams;
    use tendermint::types::{proto::params::EvidenceParams as RawEvidenceParams, time::Duration};

    #[test]
    fn evidence_params_serialize_works() {
        let params: EvidenceParams = RawEvidenceParams {
            max_age_num_blocks: 0,
            max_age_duration: Some(Duration {
                seconds: 10,
                nanos: 30,
            }),
            max_bytes: 0,
        }
        .into();

        assert_eq!(
            serde_json::to_string(&params).unwrap(),
            "{\"max_age_num_blocks\":\"0\",\"max_age_duration\":\"10000000030\",\"max_bytes\":\"0\"}"
                .to_string()
        );
    }
}
