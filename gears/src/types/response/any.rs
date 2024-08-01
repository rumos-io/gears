use crate::{
    core::Protobuf,
    types::tx::{Tx, TxMessage},
};
use core_types::{any::google::Any, errors::CoreError};
use serde::{Deserialize, Serialize};

/// This enum is used where a Tx needs to be serialized like an Any
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum AnyTx<M: TxMessage> {
    #[serde(rename = "/cosmos.tx.v1beta1.Tx")]
    Tx(Tx<M>),
}

impl<M: TxMessage> From<AnyTx<M>> for Any {
    fn from(msg: AnyTx<M>) -> Self {
        match msg {
            AnyTx::Tx(msg) => Any {
                type_url: "/cosmos.tx.v1beta1.Tx".to_string(),
                value: msg.encode_vec(),
            },
        }
    }
}

impl<M: TxMessage> TryFrom<Any> for AnyTx<M> {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.tx.v1beta1.Tx" => {
                let msg: Tx<M> = Tx::decode_vec(&value.value)
                    .map_err(|e| CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(AnyTx::Tx(msg))
            }
            _ => Err(CoreError::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
