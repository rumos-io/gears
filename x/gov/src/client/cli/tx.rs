use std::path::PathBuf;

use clap::{Args, Subcommand};
use gears::types::base::coins::Coins;

use crate::msg::{vote::VoteOption, weighted_vote::VoteOptionWeighted};

#[derive(Args, Debug, Clone)]
pub struct GovTxCli {
    #[command(subcommand)]
    pub command: GovTxCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GovTxCommands {
    Deposit(DepositCliCommand),
    Vote(VoteCliCommand),
    WeightedVote(WeightedVoteCliCommand),
    SubmitProposal(ProposalCliCommand<ProposalCliSubcommand>),
}

/// Deposit tokens for an active proposal
#[derive(Args, Debug, Clone)]
pub struct DepositCliCommand {
    pub proposal_id: u64,
    pub amount: Coins,
    // #[arg(long)]
    // pub from: AccAddress,
}

/// Vote for an active proposal
#[derive(Args, Debug, Clone)]
pub struct VoteCliCommand {
    pub proposal_id: u64,
    pub option: VoteOption,
    // #[arg(long)]
    // pub from: AccAddress,
}

/// Vote for an active proposal
#[derive(Args, Debug, Clone)]
pub struct WeightedVoteCliCommand {
    pub proposal_id: u64,
    pub options: Vec<VoteOptionWeighted>,
}

/// Submit a proposal along with an initial deposit
#[derive(Args, Debug, Clone)]
pub struct ProposalCliCommand<T: Subcommand> {
    pub initial_deposit: Coins,
    #[command(subcommand)]
    pub command: T,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ProposalCliSubcommand {
    Text(TextProposalCliCommand),
    ParamChange(ParamChangeProposalCliCommand),
}

#[derive(Args, Debug, Clone)]
pub struct TextProposalCliCommand {
    pub title: String,
    pub description: String,
}

#[derive(Args, Debug, Clone)]
pub struct ParamChangeProposalCliCommand {
    pub file: PathBuf,
}
