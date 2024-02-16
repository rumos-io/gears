use proto_messages::cosmos::ibc::protobuf::Any;
use proto_types::AccAddress;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum Message {}

//TODO: the fact that this implements proto_messages::cosmos::tx::v1beta1::Message  is not used
impl proto_messages::cosmos::tx::v1beta1::message::Message for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        vec![]
    }

    fn validate_basic(&self) -> Result<(), String> {
        Ok(())
    }

    fn type_url(&self) -> &'static str {
        "TODO"
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
    type Error = proto_messages::Error;

    fn try_from(_value: Any) -> Result<Self, Self::Error> {
        Err(proto_messages::Error::DecodeGeneral(
            "message type not recognized".into(),
        ))
    }
}
