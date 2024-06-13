use gears::types::address::AccAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: AccAddress,
    pub option: VoteOption,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VoteOption {
    Empty = 0,
    Yes,
    Abstain,
    No,
    NoWithVeto,
}
