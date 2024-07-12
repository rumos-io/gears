use std::collections::{HashMap, HashSet};

use database::Database;
use kv_store::StoreKey;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    application::keepers::params::ParamsKeeper,
    context::{InfallibleContext, InfallibleContextMut},
    params::{
        infallible_subspace, infallible_subspace_mut, ParamKind, ParamsDeserialize,
        ParamsSerialize, ParamsSubspaceKey,
    },
};

mod inner {
    pub use tendermint::types::proto::consensus::ConsensusParams;
    pub use tendermint::types::proto::params::BlockParams;
    pub use tendermint::types::proto::params::EvidenceParams;
    pub use tendermint::types::proto::params::ValidatorParams;
}

const KEY_BLOCK_PARAMS: &str = "BlockParams";
const KEY_EVIDENCE_PARAMS: &str = "EvidenceParams";
const KEY_VALIDATOR_PARAMS: &str = "ValidatorParams";

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

/// A domain ConsensusParams type that wraps domain consensus params types.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConsensusParams {
    pub block: BlockParams,
    pub evidence: EvidenceParams,
    pub validator: ValidatorParams,
    // TODO: consider to check the importance and usage
    // pub version: Option<VersionParams>
}

impl From<inner::ConsensusParams> for ConsensusParams {
    fn from(
        inner::ConsensusParams {
            block,
            evidence,
            validator,
            version: _,
        }: inner::ConsensusParams,
    ) -> Self {
        Self {
            block: block.into(),
            evidence: evidence.into(),
            validator: validator.into(),
        }
    }
}

impl ParamsSerialize for ConsensusParams {
    fn keys() -> HashSet<&'static str> {
        [KEY_BLOCK_PARAMS, KEY_EVIDENCE_PARAMS, KEY_VALIDATOR_PARAMS]
            .into_iter()
            .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut hash_map = Vec::with_capacity(3);

        let block_params =
            serde_json::to_string(&self.block).expect("conversion to json won't fail");
        hash_map.push((KEY_BLOCK_PARAMS, block_params.into_bytes()));

        let evidence_params =
            serde_json::to_string(&self.evidence).expect("conversion to json won't fail");
        hash_map.push((KEY_EVIDENCE_PARAMS, evidence_params.into_bytes()));

        let params = serde_json::to_string(&self.validator).expect("conversion to json won't fail");
        hash_map.push((KEY_VALIDATOR_PARAMS, params.into_bytes()));

        hash_map
    }
}

impl ParamsDeserialize for ConsensusParams {
    fn from_raw(fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            block: serde_json::from_slice(fields.get(KEY_BLOCK_PARAMS).unwrap()).unwrap(),
            evidence: serde_json::from_slice(fields.get(KEY_EVIDENCE_PARAMS).unwrap()).unwrap(),
            validator: serde_json::from_slice(fields.get(KEY_VALIDATOR_PARAMS).unwrap()).unwrap(),
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockParams {
    pub max_bytes: String,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub max_gas: i64,
}

impl Default for BlockParams {
    fn default() -> Self {
        // TODO:ME implement defaults
        // from sdk testing setup
        BlockParams {
            max_bytes: 200_000.to_string(),
            max_gas: 2_000_000,
        }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidatorParams {
    pub pub_key_types: Vec<String>,
}

impl Default for ValidatorParams {
    fn default() -> Self {
        // TODO: check defaults
        Self {
            pub_key_types: vec!["secp256k1".to_string()],
        }
    }
}

impl From<inner::ValidatorParams> for ValidatorParams {
    fn from(params: inner::ValidatorParams) -> ValidatorParams {
        ValidatorParams {
            pub_key_types: params.pub_key_types,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceParams {
    pub max_age_num_blocks: String,
    pub max_age_duration: Option<String>,
    pub max_bytes: String,
}

impl Default for EvidenceParams {
    fn default() -> Self {
        // TODO:ME update defaults
        // from sdk testing setup
        EvidenceParams {
            max_age_num_blocks: 302400.to_string(),
            max_age_duration: Some((504 * 3600 * SEC_TO_NANO).to_string()), // 3 weeks
            max_bytes: 10000.to_string(),
        }
    }
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
pub struct BaseAppParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> ParamsKeeper<PSK> for BaseAppParamsKeeper<PSK> {
    type Param = ConsensusParams;

    fn psk(&self) -> &PSK {
        &self.params_subspace_key
    }

    #[cfg(all(feature = "governance"))]
    fn validate(key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> bool {
        match String::from_utf8_lossy(key.as_ref()).as_ref() {
            KEY_BLOCK_PARAMS => serde_json::from_slice::<BlockParams>(value.as_ref()).is_ok(),
            KEY_EVIDENCE_PARAMS => serde_json::from_slice::<EvidenceParams>(value.as_ref()).is_ok(),
            KEY_VALIDATOR_PARAMS => {
                serde_json::from_slice::<ValidatorParams>(value.as_ref()).is_ok()
            }
            _ => false,
        }
    }
}

// TODO: add a macro to create this?
impl<PSK: ParamsSubspaceKey> BaseAppParamsKeeper<PSK> {
    pub fn set_consensus_params<DB: Database, SK: StoreKey, CTX: InfallibleContextMut<DB, SK>>(
        &self,
        ctx: &mut CTX,
        params: ConsensusParams,
    ) {
        let mut store = infallible_subspace_mut(ctx, &self.params_subspace_key);

        store.params_set(&params);
    }

    pub fn consensus_params<DB: Database, SK: StoreKey, CTX: InfallibleContext<DB, SK>>(
        &self,
        store: &CTX,
    ) -> ConsensusParams {
        let sub_store = infallible_subspace(store, &self.params_subspace_key);

        let block_params = self.block_params(store).unwrap_or_default();
        let evidence_params = sub_store
            .params_field(KEY_EVIDENCE_PARAMS, ParamKind::Bytes)
            .map(|params| {
                serde_json::from_slice(&params.bytes().expect("We sure that this is bytes"))
                    .expect("conversion from json won't fail")
            })
            .unwrap_or_default();

        let validator_params = sub_store
            .params_field(KEY_VALIDATOR_PARAMS, ParamKind::Bytes)
            .map(|params| {
                serde_json::from_slice(&params.bytes().expect("We sure that this is bytes"))
                    .expect("conversion from json won't fail")
            })
            .unwrap_or_default();

        ConsensusParams {
            block: block_params,
            evidence: evidence_params,
            validator: validator_params,
        }
    }

    pub fn block_params<DB: Database, SK: StoreKey, CTX: InfallibleContext<DB, SK>>(
        &self,
        store: &CTX,
    ) -> Option<BlockParams> {
        let sub_store = infallible_subspace(store, &self.params_subspace_key);

        serde_json::from_slice(
            &sub_store
                .params_field(KEY_BLOCK_PARAMS, ParamKind::Bytes)?
                .bytes()
                .expect("We sure that this is bytes"),
        )
        .ok()
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
