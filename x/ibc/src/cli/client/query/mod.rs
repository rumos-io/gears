use clap::{Args, Subcommand};
use tendermint::informal::block::Height;

pub mod client_params;
pub mod client_state;
pub mod client_states;
pub mod client_status;
pub mod consensus_heights;
pub mod consensus_state;
pub mod consensus_states;
pub mod query_header;
pub mod self_consensus_state;

#[derive(Args, Debug)]
pub struct IbcQueryCli {
    #[command(subcommand)]
    command: IbcCommands,
}

#[derive(Subcommand, Debug)]
pub enum IbcCommands {
    ClientParams(client_params::CliClientParams),
    ClientState(client_state::CliClientParams),
    ClientStates(client_states::CliClientParams),
    ClientStatus(client_status::CliClientParams),
    ConsensusState(consensus_state::CliClientParams),
    ConsensusStates(consensus_states::CliClientParams),
    ConsensusStateHeights(consensus_heights::CliClientParams),
    Header(query_header::CliClientParams),
    SelfState(self_consensus_state::CliClientParams),
}

pub async fn run_ibc_query_command(
    args: IbcQueryCli,
    node: &str,
    height: Option<Height>,
) -> anyhow::Result<String> {
    match args.command {
        IbcCommands::ClientParams(args) => client_params::query_command_handler(args, node, height).await,
        IbcCommands::ClientState(args) => client_state::query_command_handler(args, node, height).await,
        IbcCommands::ClientStates(args) => client_states::query_command_handler(args, node, height).await,
        IbcCommands::ClientStatus(args) => client_status::query_command_handler(args, node, height).await,
        IbcCommands::ConsensusState(args) => {
            consensus_state::query_command_handler(args, node, height).await
        }
        IbcCommands::ConsensusStates(args) => {
            consensus_states::query_command_handler(args, node, height).await
        }
        IbcCommands::ConsensusStateHeights(args) => {
            consensus_heights::query_command_handler(args, node, height).await
        }
        IbcCommands::Header(args) => query_header::query_command_handler(args, node, height).await,
        IbcCommands::SelfState(args) => {
            self_consensus_state::query_command_handler(args, node, height)
        }
    }
}
