use gears::derive::AppMessage;
use serde::Serialize;

use crate::{MsgFundCommunityPool, MsgSetWithdrawAddr, MsgWithdrawDelegatorReward};

#[derive(Debug, Clone, Serialize, AppMessage)]
pub enum Message {
    #[serde(rename = "/cosmos.distribution.v1beta1.WithdrawRewards")]
    #[msg(url(path = MsgWithdrawDelegatorReward::TYPE_URL))]
    WithdrawRewards(MsgWithdrawDelegatorReward),
    #[serde(rename = "/cosmos.distribution.v1beta1.SetWithdrawAddr")]
    #[msg(url(path = MsgSetWithdrawAddr::TYPE_URL))]
    SetWithdrawAddr(MsgSetWithdrawAddr),
    #[serde(rename = "/cosmos.distribution.v1beta1.FundCommunityPool")]
    #[msg(url(path = MsgFundCommunityPool::TYPE_URL))]
    FundCommunityPool(MsgFundCommunityPool),
}
