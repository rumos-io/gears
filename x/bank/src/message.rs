use bytes::Bytes;
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};
use proto_messages::cosmos::bank::v1beta1::MsgSend;
use proto_types::AccAddress;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum Message {
    Send(MsgSend),
}

//TODO: the fact that this implements proto_messages::cosmos::tx::v1beta1::Message  is not used
impl proto_messages::cosmos::tx::v1beta1::Message for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Message::Send(msg) => return vec![&msg.from_address],
        }
    }

    fn validate_basic(&self) -> Result<(), String> {
        match &self {
            Message::Send(_) => Ok(()),
        }
    }
}

// #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// #[serde(tag = "@type")]
// pub enum Msg {
//     #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
//     Send(MsgSend),
// }

//impl Msg {
//     pub fn get_signers(&self) -> Vec<&AccAddress> {
//         match &self {
//             Msg::Send(msg) => return vec![&msg.from_address],
//         }
//     }

//     pub fn validate_basic(&self) -> Result<(), Error> {
//         match &self {
//             Msg::Send(_) => Ok(()),
//         }
//     }
// }

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Send(msg) => Any {
                type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                value: msg.encode_vec(),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = proto_messages::Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.bank.v1beta1.MsgSend" => {
                let msg = MsgSend::decode::<Bytes>(value.value.clone().into()).unwrap();
                Ok(Message::Send(msg))
            }
            _ => Err(proto_messages::Error::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
