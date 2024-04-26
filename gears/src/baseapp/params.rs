use serde::{Deserialize, Serialize};
use store_crate::database::Database;
use store_crate::{
    QueryableMultiKVStore, ReadPrefixStore, StoreKey, TransactionalMultiKVStore, WritePrefixStore,
};
use tendermint::types::proto::consensus::ConsensusParams;

use crate::params::{Keeper, ParamsSubspaceKey};

mod inner {
    pub use tendermint::types::proto::params::BlockParams;
    pub use tendermint::types::proto::params::EvidenceParams;
    pub use tendermint::types::proto::params::ValidatorParams;
}

const KEY_BLOCK_PARAMS: [u8; 11] = [066, 108, 111, 099, 107, 080, 097, 114, 097, 109, 115]; // "BlockParams"
const KEY_EVIDENCE_PARAMS: [u8; 14] = [
    069, 118, 105, 100, 101, 110, 099, 101, 080, 097, 114, 097, 109, 115,
]; // "EvidenceParams"
const KEY_VALIDATOR_PARAMS: [u8; 15] = [
    086, 097, 108, 105, 100, 097, 116, 111, 114, 080, 097, 114, 097, 109, 115,
]; // "ValidatorParams"

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

#[derive(Serialize, Deserialize)]
pub struct BlockParams {
    pub max_bytes: String,
    pub max_gas: String,
}

impl From<inner::BlockParams> for BlockParams {
    fn from(params: inner::BlockParams) -> BlockParams {
        BlockParams {
            max_bytes: params.max_bytes.to_string(),
            max_gas: {
                if params.max_gas < -1 {
                    panic!("Invalid max block gas")
                } else if params.max_gas == -1 {
                    0.to_string()
                } else {
                    params.max_gas.to_string()
                }
            },
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
    pub params_keeper: Keeper<SK, PSK>,
    pub params_subspace_key: PSK,
}

// TODO: add a macro to create this?
impl<SK: StoreKey, PSK: ParamsSubspaceKey> BaseAppParamsKeeper<SK, PSK> {
    pub fn set_consensus_params<DB: Database, KV: TransactionalMultiKVStore<DB, SK>>(
        &self,
        kv_store: &mut KV,
        params: ConsensusParams,
    ) {
        // let store = ctx.get_mutable_kv_store(crate::store::Store::Params);
        // let mut store = store.get_mutable_prefix_store(SUBSPACE_NAME.into());

        let mut store = self
            .params_keeper
            .raw_subspace_mut(kv_store, &self.params_subspace_key);

        if let Some(params) = params.block {
            let block_params = serde_json::to_string(&BlockParams::from(params))
                .expect("conversion to json won't fail");
            store.set(KEY_BLOCK_PARAMS, block_params.into_bytes());
        }

        if let Some(params) = params.evidence {
            let evidence_params = serde_json::to_string(&EvidenceParams::from(params))
                .expect("conversion to json won't fail");
            store.set(KEY_EVIDENCE_PARAMS, evidence_params.into_bytes());
        }

        if let Some(params) = params.validator {
            let params = serde_json::to_string(&ValidatorParams::from(params))
                .expect("conversion to json won't fail");
            store.set(KEY_VALIDATOR_PARAMS, params.into_bytes());
        }
    }

    pub fn block_params<DB: Database, KV: QueryableMultiKVStore<DB, SK>>(
        &self,
        store: &KV,
    ) -> Option<BlockParams> {
        let sub_store = self
            .params_keeper
            .raw_subspace(store, &self.params_subspace_key);

        let params = sub_store.get(&KEY_BLOCK_PARAMS);

        if let Some(params) = params {
            let block_params: BlockParams =
                serde_json::from_slice(&params).expect("conversion from json won't fail");

            Some(block_params)
        } else {
            None
        }
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
