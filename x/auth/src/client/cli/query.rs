use anyhow::Result;
use clap::{Args, Subcommand};

use gears::client::query::run_query;

use tendermint::informal::block::Height;

use proto_messages::cosmos::{
    auth::v1beta1::{QueryAccountRequest, QueryAccountResponse},
    ibc::{auth::RawQueryAccountResponse, protobuf::Protobuf},
};
use proto_types::AccAddress;

#[derive(Args, Debug)]
pub struct AuthQueryCli {
    #[command(subcommand)]
    command: AuthCommands,
}

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Query for account by address
    Account {
        /// address
        address: AccAddress,
    },
}

pub async fn run_auth_query_command(
    args: AuthQueryCli,
    node: &str,
    height: Option<Height>,
) -> Result<String> {
    match args.command {
        AuthCommands::Account { address } => {
            let query = QueryAccountRequest { address };

            let res = run_query::<QueryAccountResponse, RawQueryAccountResponse>(
                query.encode_vec(),
                "/cosmos.auth.v1beta1.Query/Account".into(),
                node,
                height,
            )
            .await?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
    }
}
