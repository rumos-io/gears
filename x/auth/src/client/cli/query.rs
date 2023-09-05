use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

use gears::client::query::run_query;
use ibc_proto::protobuf::Protobuf;

use tendermint_informal::block::Height;

use ibc_proto::cosmos::auth::v1beta1::QueryAccountResponse as RawQueryAccountResponse;
use proto_messages::cosmos::auth::v1beta1::{QueryAccountRequest, QueryAccountResponse};
use proto_types::AccAddress;

pub fn get_auth_query_command() -> Command {
    Command::new("auth")
        .about("Querying commands for the auth module")
        .subcommand(
            Command::new("account")
                .about("Query for account by address")
                .arg(
                    Arg::new("address")
                        .required(true)
                        .value_parser(clap::value_parser!(AccAddress)),
                ),
        )
        .subcommand_required(true)
}

pub fn run_auth_query_command(
    matches: &ArgMatches,
    node: &str,
    height: Option<Height>,
) -> Result<String> {
    match matches.subcommand() {
        Some(("account", sub_matches)) => {
            let address = sub_matches
                .get_one::<AccAddress>("address")
                .expect("address argument is required preventing `None`")
                .to_owned();

            let query = QueryAccountRequest { address };

            let res = run_query::<QueryAccountResponse, RawQueryAccountResponse>(
                query.encode_vec(),
                "/cosmos.auth.v1beta1.Query/Account".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
