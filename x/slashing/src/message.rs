use gears::derive::AppMessage;
use serde::Serialize;

use crate::MsgUnjail;

#[derive(Debug, Clone, Serialize, AppMessage)]
pub enum Message {
    #[serde(rename = "/cosmos.slashing.v1beta1.Unjail")]
    #[msg(url(path = MsgUnjail::TYPE_URL))]
    Unjail(MsgUnjail),
}
