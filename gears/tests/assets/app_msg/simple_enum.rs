fn main() {}

use gears::{derive::AppMessage, types::msg::send::MsgSend};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, AppMessage)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    #[msg(url(path = MsgSend::TYPE_URL))]
    Send(MsgSend),
}
