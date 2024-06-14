use deposit::MsgDeposit;
use gears::derive::RoutingMessage;
use serde::Serialize;
use vote::MsgVote;
use weighted_vote::MsgVoteWeighted;

pub mod deposit;
pub mod proposal;
pub mod vote;
pub mod weighted_vote;

#[derive(Debug, Clone, Serialize, RoutingMessage)]
pub enum GovMsg {
    #[gears(url = "/cosmos.gov.v1beta1/MsgDeposit")]
    Deposit(MsgDeposit),
    #[gears(url = "/cosmos.gov.v1beta1/MsgVote")]
    Vote(MsgVote),
    #[gears(url = "/cosmos.gov.v1beta1/MsgVoteWeighted")]
    Weighted(MsgVoteWeighted),
}
