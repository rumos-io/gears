use crate::{CreateValidator, DelegateMsg, EditValidator, RedelegateMsg, UndelegateMsg};
use gears::derive::AppMessage;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, AppMessage)]
#[serde(tag = "@type")]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    #[serde(rename = "/cosmos.staking.v1beta1.MsgCreateValidator")]
    #[msg(url(path = CreateValidator::TYPE_URL))]
    CreateValidator(CreateValidator),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgEditValidator")]
    #[msg(url(path = EditValidator::TYPE_URL))]
    EditValidator(EditValidator),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgDelegate")]
    #[msg(url(path = DelegateMsg::TYPE_URL))]
    Delegate(DelegateMsg),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgBeginRedelegate")]
    #[msg(url(path = RedelegateMsg::TYPE_URL))]
    Redelegate(RedelegateMsg),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgUndelegate")]
    #[msg(url(path = UndelegateMsg::TYPE_URL))]
    Undelegate(UndelegateMsg),
}
