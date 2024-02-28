use auth::signing::renderer::value_renderer::ValueRenderer;
use bytes::Bytes;
use proto_messages::cosmos::bank::v1beta1::MsgSend;
use proto_messages::cosmos::tx::v1beta1::screen::Screen;
use proto_messages::cosmos::tx::v1beta1::tx_metadata::Metadata;
use proto_messages::{any::Any, cosmos::ibc::protobuf::Protobuf};
use proto_types::AccAddress;
use proto_types::Denom;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    Send(MsgSend),
}

impl ValueRenderer for Message {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        match self {
            Message::Send(msg) => msg.format(get_metadata),
        }
    }
}

impl proto_messages::cosmos::tx::v1beta1::message::Message for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Message::Send(msg) => vec![&msg.from_address],
        }
    }

    fn validate_basic(&self) -> Result<(), String> {
        match &self {
            Message::Send(_) => Ok(()),
        }
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::Send(_) => "/cosmos.bank.v1beta1.MsgSend",
        }
    }
}

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
                let msg = MsgSend::decode::<Bytes>(value.value.clone().into())
                    .map_err(|e| proto_messages::Error::DecodeProtobuf(e.to_string()))?;
                Ok(Message::Send(msg))
            }
            _ => Err(proto_messages::Error::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
