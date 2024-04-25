use clap::{Args, Subcommand};
use gears::core::address::AccAddress;
use ibc::clients::tendermint::client_state::ClientState;
use ibc::clients::tendermint::consensus_state::ConsensusState;
use std::fs::File;
use std::io::Read;

use crate::ics02_client::message::MsgCreateClient;

#[derive(Args, Debug, Clone)]
pub struct ClientTxCli {
    #[command(subcommand)]
    pub command: ClientCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ClientCommands {
    /// Create a new IBC client with the specified client state and consensus state
    Create {
        /// JSON input or path to .json file containing the client state
        client_state: String,
        /// JSON input or path to .json file containing the consensus state
        consensus_state: String,
    },
}

pub(crate) fn tx_command_handler(
    args: ClientTxCli,
    from_address: AccAddress,
) -> anyhow::Result<crate::message::Message> {
    match args.command {
        ClientCommands::Create {
            client_state,
            consensus_state,
        } => {
            let client_state_result = serde_json::from_str::<ClientState>(&client_state);

            let client_state = if let Ok(client_state) = client_state_result {
                client_state
            } else {
                let mut buffer = Vec::<u8>::new();
                File::open(client_state)?.read_to_end(&mut buffer)?;
                serde_json::from_slice(&buffer)?
            };

            let consensus_state_result = serde_json::from_str::<ConsensusState>(&consensus_state);
            let consensus_state = if let Ok(consensus_state) = consensus_state_result {
                consensus_state
            } else {
                let mut buffer = Vec::<u8>::new();
                File::open(consensus_state)?.read_to_end(&mut buffer)?;
                serde_json::from_slice(&buffer)?
            };

            let raw_msg = MsgCreateClient {
                client_state,
                consensus_state,
                signer: from_address,
            };

            Ok(crate::message::Message::ClientCreate(raw_msg))
        }
    }
}
