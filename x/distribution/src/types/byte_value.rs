use gears::core::Protobuf;
use prost::Message;
use serde::{Deserialize, Serialize};

// TODO: maybe move to gears
#[derive(Clone, PartialEq, Message, Deserialize, Serialize)]
pub struct ByteValue {
    #[prost(bytes, tag = "1")]
    pub value: Vec<u8>,
}

impl Protobuf<ByteValue> for ByteValue {}
