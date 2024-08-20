use crate::types::MsgSubmitEvidence;
use gears::{
    core::{any::google::Any, Protobuf},
    types::{address::AccAddress, tx::TxMessage},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    #[serde(rename = "/cosmos.evidence.v1beta1.SubmitEvidence")]
    SubmitEvidence(MsgSubmitEvidence),
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match self {
            Message::SubmitEvidence(msg) => vec![&msg.submitter],
        }
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::SubmitEvidence(_) => "/cosmos.evidence.v1beta1.SubmitEvidence",
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::SubmitEvidence(msg) => Any {
                type_url: "/cosmos.evidence.v1beta1.SubmitEvidence".to_string(),
                value: msg.encode_vec(),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.evidence.v1beta1.SubmitEvidence" => {
                let msg = MsgSubmitEvidence::decode_vec(&value.value)
                    .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                Ok(Message::SubmitEvidence(msg))
            }
            _ => Err(gears::core::errors::CoreError::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
