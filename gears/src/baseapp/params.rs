use database::DB;
use serde::{Deserialize, Serialize};
use tendermint_proto::{abci::BlockParams as RawBlockParams, abci::ConsensusParams};

use tendermint_proto::types::EvidenceParams as RawEvidenceParams;
use tendermint_proto::types::ValidatorParams as RawValidatorParams;

use crate::types::Context;

const KEY_BLOCK_PARAMS: [u8; 11] = [066, 108, 111, 099, 107, 080, 097, 114, 097, 109, 115]; // "BlockParams"
const KEY_EVIDENCE_PARAMS: [u8; 14] = [
    069, 118, 105, 100, 101, 110, 099, 101, 080, 097, 114, 097, 109, 115,
]; // "EvidenceParams"
const KEY_VALIDATOR_PARAMS: [u8; 15] = [
    086, 097, 108, 105, 100, 097, 116, 111, 114, 080, 097, 114, 097, 109, 115,
]; // "ValidatorParams"

const SUBSPACE_NAME: &str = "baseapp/";

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

impl From<RawBlockParams> for BlockParams {
    fn from(params: RawBlockParams) -> BlockParams {
        BlockParams {
            max_bytes: params.max_bytes.to_string(),
            max_gas: params.max_gas.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ValidatorParams {
    pub pub_key_types: Vec<String>,
}

impl From<RawValidatorParams> for ValidatorParams {
    fn from(params: RawValidatorParams) -> ValidatorParams {
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

impl From<RawEvidenceParams> for EvidenceParams {
    fn from(params: RawEvidenceParams) -> EvidenceParams {
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

pub fn set_consensus_params<T: DB>(ctx: &mut Context<T>, params: ConsensusParams) {
    let store = ctx.get_mutable_kv_store(crate::store::Store::Params);
    let mut store = store.get_mutable_prefix_store(SUBSPACE_NAME.into());

    if let Some(params) = params.block {
        let block_params = serde_json::to_string(&BlockParams::from(params))
            .expect("conversion to json won't fail");
        store.set(KEY_BLOCK_PARAMS.into(), block_params.into_bytes());
    }

    if let Some(params) = params.evidence {
        let evidence_params = serde_json::to_string(&EvidenceParams::from(params))
            .expect("conversion to json won't fail");
        store.set(KEY_EVIDENCE_PARAMS.into(), evidence_params.into_bytes());
    }

    if let Some(params) = params.validator {
        let params = serde_json::to_string(&ValidatorParams::from(params))
            .expect("conversion to json won't fail");
        store.set(KEY_VALIDATOR_PARAMS.into(), params.into_bytes());
    }
}

#[cfg(test)]
mod tests {

    use tendermint_proto::google::protobuf::Duration;
    use tendermint_proto::types::EvidenceParams as RawEvidenceParams;

    use super::*;

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
