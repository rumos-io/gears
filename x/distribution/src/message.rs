use gears::core::any::google::Any;
use gears::types::address::AccAddress;
use gears::types::tx::TxMessage;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum Message {
    #[serde(rename = "/cosmos.distribution.v1beta1.Todo")]
    Todo(String /* TODO */),
}

impl TxMessage for Message {
    fn get_signers(&self) -> Vec<&AccAddress> {
        match self {
            Message::Todo(_msg) => vec![/* TODO */],
        }
    }

    fn type_url(&self) -> &'static str {
        match self {
            Message::Todo(_) => "/cosmos.distribution.v1beta1.Todo",
        }
    }
}

impl From<Message> for Any {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Todo(msg) => Any {
                type_url: "/cosmos.distribution.v1beta1.Todo".to_string(),
                value: msg.into_bytes(),
                // value: msg.encode_vec(),
            },
        }
    }
}

impl TryFrom<Any> for Message {
    type Error = gears::core::errors::CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "/cosmos.distribution.v1beta1.Todo" => {
                // let msg = String /* TODO */::decode::<Bytes>(value.value.clone().into())
                //     .map_err(|e| gears::core::errors::CoreError::DecodeProtobuf(e.to_string()))?;
                let msg = String::from_utf8(value.value.to_vec()).unwrap();
                Ok(Message::Todo(msg))
            }
            _ => Err(gears::core::errors::CoreError::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}
