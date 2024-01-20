use gears_derive::Message;
use ibc_proto::google::protobuf::Any;
use proto_types::AccAddress;
use serde::Serialize;

#[derive(Debug, Clone, Message, Serialize)]
#[serde(untagged)]
pub enum Message {
    #[gears(url = "/cosmos.bank.v1beta1")]
    Bank(bank::Message),
}
