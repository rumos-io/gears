use std::{fs::File, io::Read};

use gears::{application::handlers::client::TxHandler, types::address::AccAddress};

use crate::{
    client::cli::tx::{
        DepositCliCommand, GovTxCli, GovTxCommands, ParamChangeProposalCliCommand,
        ProposalCliCommand, ProposalCliSubcommand, TextProposalCliCommand, VoteCliCommand,
        WeightedVoteCliCommand,
    },
    msg::{
        deposit::Deposit, proposal::MsgSubmitProposal, vote::Vote, weighted_vote::MsgVoteWeighted,
        GovMsg,
    },
    submission::{param::RawParameterChangeProposal, text::TextProposal},
};

use super::GovClientHandler;

impl TxHandler for GovClientHandler {
    type Message = GovMsg;

    type TxCommands = GovTxCli;

    fn prepare_tx(
        &self,
        command: Self::TxCommands,
        from_address: AccAddress,
    ) -> anyhow::Result<Self::Message> {
        match command.command {
            GovTxCommands::Deposit(DepositCliCommand {
                proposal_id,
                amount,
            }) => Ok(GovMsg::Deposit(Deposit {
                proposal_id,
                depositor: from_address,
                amount,
            })),
            GovTxCommands::Vote(VoteCliCommand {
                proposal_id,
                option,
            }) => Ok(GovMsg::Vote(Vote {
                proposal_id,
                voter: from_address,
                option,
            })),
            GovTxCommands::WeightedVote(WeightedVoteCliCommand {
                proposal_id,
                options,
            }) => Ok(GovMsg::Weighted(MsgVoteWeighted {
                proposal_id,
                voter: from_address,
                options,
            })),
            GovTxCommands::SubmitProposal(ProposalCliCommand {
                initial_deposit,
                command,
            }) => match command {
                ProposalCliSubcommand::Text(TextProposalCliCommand { title, description }) => {
                    Ok(GovMsg::Proposal(MsgSubmitProposal {
                        content: TextProposal { title, description }.into(),
                        initial_deposit,
                        proposer: from_address,
                    }))
                }
                ProposalCliSubcommand::ParamChange(ParamChangeProposalCliCommand { file }) => {
                    let mut buf = String::new();
                    File::open(file)?.read_to_string(&mut buf)?;

                    let proposal = serde_json::from_str::<RawParameterChangeProposal>(&buf)?;

                    Ok(GovMsg::Proposal(MsgSubmitProposal {
                        content: proposal.into(),
                        initial_deposit,
                        proposer: from_address,
                    }))
                }
            },
        }
    }
}
