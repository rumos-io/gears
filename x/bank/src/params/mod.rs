use gears::application::keepers::params::ParamsKeeper;
use gears::derive::Protobuf;
use gears::extensions::corruption::UnwrapCorrupt;
use gears::params::{ParamKind, ParamsDeserialize, ParamsSerialize, ParamsSubspaceKey};
use gears::types::denom::Denom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const KEY_SEND_ENABLED: &str = "SendEnabled";
const KEY_DEFAULT_SEND_ENABLED: &str = "DefaultSendEnabled";

mod inner {
    pub use ibc_proto::cosmos::bank::v1beta1::Params;
    pub use ibc_proto::cosmos::bank::v1beta1::SendEnabled;
}

mod environment;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Protobuf)]
#[proto(raw = "inner::SendEnabled")]
pub struct SendEnabled {
    pub denom: Denom,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Protobuf)]
#[proto(raw = "inner::Params")]
pub struct BankParams {
    #[proto(repeated)]
    pub send_enabled: Vec<SendEnabled>,
    pub default_send_enabled: bool,
}

pub const DEFAULT_PARAMS: BankParams = BankParams {
    send_enabled: vec![],
    default_send_enabled: true,
};

impl Default for BankParams {
    fn default() -> Self {
        DEFAULT_PARAMS.clone()
    }
}

impl ParamsSerialize for BankParams {
    fn keys() -> HashSet<&'static str> {
        [KEY_SEND_ENABLED, KEY_DEFAULT_SEND_ENABLED]
            .into_iter()
            .collect()
    }

    fn to_raw(&self) -> Vec<(&'static str, Vec<u8>)> {
        vec![
            (
                KEY_DEFAULT_SEND_ENABLED,
                self.default_send_enabled.to_string().into_bytes(),
            ),
            // TODO: if params are missing in the cosmos SDK (e.g the send_enabled field is missing from the genesis json file)
            // then they are set to "null" i.e. [110, 117, 108, 108] when stored. This is different to the Gears behaviour
            // which would fail to parse if e.g. the send enabled field is missing
            (
                KEY_SEND_ENABLED,
                serde_json::to_vec(&self.send_enabled)
                    .expect("conversion of domain types won't fail"),
            ),
        ]
    }
}

impl ParamsDeserialize for BankParams {
    fn from_raw(mut fields: HashMap<&'static str, Vec<u8>>) -> Self {
        Self {
            default_send_enabled: ParamKind::Bool
                .parse_param(fields.remove(KEY_DEFAULT_SEND_ENABLED).unwrap_or_corrupt())
                .boolean()
                .unwrap_or_corrupt(),
            send_enabled: serde_json::from_slice(
                &ParamKind::Bytes
                    .parse_param(fields.remove(KEY_SEND_ENABLED).unwrap_or_corrupt())
                    .bytes()
                    .unwrap_or_corrupt(),
            )
            .unwrap_or_corrupt(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BankParamsKeeper<PSK: ParamsSubspaceKey> {
    pub params_subspace_key: PSK,
}

impl<PSK: ParamsSubspaceKey> ParamsKeeper<PSK> for BankParamsKeeper<PSK> {
    type Param = BankParams;

    fn psk(&self) -> &PSK {
        &self.params_subspace_key
    }

    fn validate(key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> bool {
        match String::from_utf8_lossy(key.as_ref()).as_ref() {
            KEY_SEND_ENABLED => ParamKind::Bool
                .parse_param(value.as_ref().to_vec())
                .boolean()
                .is_some(),
            KEY_DEFAULT_SEND_ENABLED => false, // add logic when we start setting this key
            _ => false,
        }
    }
}
