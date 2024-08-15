use crate::{message::Message, types::MsgSubmitEvidence};
use clap::{Args, Subcommand};
use gears::types::{address::AccAddress, tx::Messages};

#[derive(Args, Debug, Clone)]
pub struct EvidenceTxCli {
    #[command(subcommand)]
    pub command: EvidenceCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum EvidenceCommands {
    /// Submit returns the top-level evidence submission command handler.
    /// All concrete evidence submission child command handlers should be registered
    /// under this command.
    Submit { evidence: String },
}

pub fn run_tx_command(
    args: EvidenceTxCli,
    from_address: AccAddress,
) -> anyhow::Result<Messages<Message>> {
    match &args.command {
        EvidenceCommands::Submit { evidence } => {
            let evidence = serde_json::from_str(evidence)?;
            Ok(Message::SubmitEvidence(MsgSubmitEvidence {
                submitter: from_address.clone(),
                evidence,
            })
            .into())
        }
    }
}
