use clap::{Args, Subcommand};
use gears::types::address::AccAddress;

use crate::{query::request::ParamsQuery, types::proposal::ProposalStatus};

#[derive(Args, Debug, Clone)]
pub struct GovQueryCli {
    #[command(subcommand)]
    pub command: GovQueryCliCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GovQueryCliCommands {
    Deposit {
        proposal_id: u64,
        depositor: AccAddress,
    },
    Deposits {
        proposal_id: u64,
    },
    Params {
        kind: ParamsQuery,
    },
    AllParams,
    Proposal {
        proposal_id: u64,
    },
    Proposals {
        voter: AccAddress,
        depositor: AccAddress,
        status: ProposalStatus,
    },
    Tally {
        proposal_id: u64,
    },
    Vote {
        proposal_id: u64,
        voter: AccAddress,
    },
    Votes {
        proposal_id: u64,
    },
    Proposer {
        proposal_id: u64,
    },
}
