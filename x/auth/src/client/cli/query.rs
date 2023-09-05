use anyhow::Result;
use clap::{Args, Subcommand};

use gears::client::query::run_query;
use ibc_proto::protobuf::Protobuf;

use tendermint_informal::block::Height;

use ibc_proto::cosmos::auth::v1beta1::QueryAccountResponse as RawQueryAccountResponse;
use proto_messages::cosmos::auth::v1beta1::{QueryAccountRequest, QueryAccountResponse};
use proto_types::AccAddress;

#[derive(Args, Debug)]
pub struct QueryCli {
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

pub fn run_auth_query_command(
    args: QueryCli,
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
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
    }
}
