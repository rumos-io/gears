use gears::core::any::google::Any;
use gears::types::address::AccAddress;
use gears::types::tx::TxMessage;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum Message {}

// You can't execute this methods with empty enum
impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        unreachable!()
    }

    fn validate_basic(&self) -> Result<(), String> {
        unreachable!()
    }

    fn type_url(&self) -> &'static str {
        unreachable!()
    }
}

impl From<Message> for Any {
    fn from(_msg: Message) -> Self {
        Any {
            type_url: "/cosmos.auth.v1beta1".to_string(),
            value: vec![],
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::CoreError;

    fn try_from(_value: Any) -> Result<Self, Self::Error> {
        Err(gears::core::errors::CoreError::DecodeGeneral(
            "message type not recognized".into(),
        ))
    }
}
