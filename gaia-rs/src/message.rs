use ibc_proto::google::protobuf::Any;
use proto_messages::cosmos::tx::v1beta1::Message as SDKMessage;
use proto_types::AccAddress;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Message {
    Bank(bank::Message),
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Bank(msg) => msg.into(),
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = proto_messages::Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        if value.type_url.starts_with("/cosmos.bank") {
            Ok(Message::Bank(Any::try_into(value)?))
        } else {
            Err(proto_messages::Error::DecodeGeneral(
                "message type not recognized".into(),
            ))
        }
    }
}

impl SDKMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match self {
            Message::Bank(msg) => msg.get_signers(),
        }
    }

    fn validate_basic(&self) -> std::result::Result<(), String> {
        match self {
            Message::Bank(msg) => msg.validate_basic(),
        }
    }
}
