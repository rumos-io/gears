pub mod proto;
use gears::derive::RoutingMessage;
use proto::deposit::MsgDeposit;
use serde::Serialize;

pub mod deposit;
pub mod proposal;
pub mod vote;
pub mod weighted_vote;

#[derive(Debug, Clone, Serialize, RoutingMessage)]
pub enum GovMsg {
    #[gears(url = "/cosmos.gov.v1beta1/MsgDeposit")]
    Deposit(MsgDeposit),
    // Vote(Vote),
    // VoteWeighted(VoteWeighted),
}
