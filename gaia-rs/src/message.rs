use gears_derive::RoutingMessage;
use proto_types::AccAddress;
use serde::Serialize;

#[derive(Debug, Clone, RoutingMessage, Serialize)]
#[serde(untagged)]
pub enum Message {
    #[gears(url = "/cosmos.bank.v1beta1")]
    Bank(bank::Message),
    #[gears(url = "/ibc.core.client.v1")]
    Ibc(ibc::message::Message),
}
