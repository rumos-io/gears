use crate::types::MsgSubmitEvidence;
use gears::derive::AppMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, AppMessage)]
pub enum Message {
    #[serde(rename = "/cosmos.evidence.v1beta1.SubmitEvidence")]
    #[msg(url(path = MsgSubmitEvidence::TYPE_URL))]
    SubmitEvidence(MsgSubmitEvidence),
}
