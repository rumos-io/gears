use deposit::MsgDeposit;
use gears::{
    derive::RoutingMessage,
    signing::{
        handler::MetadataGetter,
        renderer::value_renderer::{RenderError, ValueRenderer},
    },
    types::rendering::screen::Screen,
};
use proposal::MsgSubmitProposal;
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
    #[gears(url = "/cosmos.gov.v1beta1/MsgSubmitProposal")]
    Proposal(MsgSubmitProposal),
}

impl ValueRenderer for GovMsg {
    fn format<MG: MetadataGetter>(&self, _: &MG) -> Result<Vec<Screen>, RenderError> {
        Err(RenderError::NotImplemented)
    }
}
