use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command, Subcommand};
use ibc_proto::cosmos::auth::v1beta1::QueryAccountResponse as RawQueryAccountResponse;
use ibc_proto::{cosmos::tx::v1beta1::TxRaw, protobuf::Protobuf};
use ibc_relayer::keyring::{Secp256k1KeyPair, SigningKeyPair};
use prost::Message;
use proto_messages::cosmos::{
    auth::v1beta1::{QueryAccountRequest, QueryAccountResponse},
    base::v1beta1::SendCoins,
    tx::v1beta1::{Fee, Message as SDKMessage, TxBody},
};
use proto_types::AccAddress;
use tendermint_informal::chain::Id;
use tendermint_rpc::{Client, HttpClient};
use tokio::runtime::Runtime;

use crate::{
    crypto::{create_signed_transaction, SigningInfo},
    utils::get_default_home_dir,
};

use super::{keys::key_store::DiskStore, query::run_query};

pub fn get_tx_command<TxSubcommand: Subcommand>(app_name: &str) -> Command {
    let cli = Command::new("tx")
        .about("Transaction subcommands")
        .subcommand_required(true)
        .arg(
            arg!(--node)
                .help("<host>:<port> to Tendermint RPC interface for this chain")
                .default_value("http://localhost:26657")
                .action(ArgAction::Set)
                .global(true),
        )
        .arg(
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir(app_name)
                        .unwrap_or_default()
                        .display()
                        .to_string()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf))
                .global(true),
        )
        .arg(
            Arg::new("from_key")
                .required(true)
                .help("From key")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("chain-id")
                .long("chain-id")
                .default_value("test-chain")
                .action(ArgAction::Set)
                .value_parser(value_parser!(Id))
                .global(true),
        )
        .arg(
            Arg::new("fee")
                .long("fee")
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(SendCoins))
                .global(true),
        );
    TxSubcommand::augment_subcommands(cli)
}

pub fn run_tx_command<TxSubcommand: Subcommand, TxCmdHandler, M>(
    matches: &ArgMatches,
    app_name: &str,
    tx_command_handler: TxCmdHandler,
) -> Result<()>
where
    M: SDKMessage,
    TxCmdHandler: FnOnce(TxSubcommand, AccAddress) -> Result<M>,
{
    let node = matches
        .get_one::<String>("node")
        .expect("Node arg has a default value so this cannot be `None`.")
        .as_str();

    let default_home_directory = get_default_home_dir(app_name);
    let home = matches
        .get_one::<PathBuf>("home")
        .or(default_home_directory.as_ref())
        .ok_or(anyhow!(
            "Home argument not provided and OS does not provide a default home directory"
        ))?
        .to_owned();

    let from = matches
        .get_one::<String>("from_key")
        .expect("from address argument is required preventing `None`")
        .to_owned();

    let chain_id = matches
        .get_one::<Id>("chain-id")
        .expect("has a default value so will never be None")
        .clone();

    let fee_amount = matches.get_one::<SendCoins>("fee").cloned();

    let key_store: DiskStore<Secp256k1KeyPair> = DiskStore::new(home)?;
    let key = key_store.get_key(&from)?;

    let args = TxSubcommand::from_arg_matches(matches).unwrap(); // TODO: remove unwrap
    let message = tx_command_handler(args, AccAddress::from_str(&key.account())?)?;

    let fee = Fee {
        amount: fee_amount,
        gas_limit: 100000000, //TODO: remove hard coded gas limit
        payer: None,          //TODO: remove hard coded payer
        granter: "".into(),   //TODO: remove hard coded granter
    };

    let account = get_account_latest(AccAddress::from_str(&key.account())?, node)?;

    let signing_info = SigningInfo {
        key,
        sequence: account.account.get_sequence(),
        account_number: account.account.get_account_number(),
    };

    let tx_body = TxBody {
        messages: vec![message],
        memo: "".into(),                        // TODO: remove hard coded
        timeout_height: 0,                      // TODO: remove hard coded
        extension_options: vec![],              // TODO: remove hard coded
        non_critical_extension_options: vec![], // TODO: remove hard coded
    };

    let tip = None; //TODO: remove hard coded

    let raw_tx = create_signed_transaction(vec![signing_info], tx_body, fee, tip, chain_id);

    let client = HttpClient::new(node)?;
    Runtime::new()
        .expect("unclear why this would ever fail")
        .block_on(broadcast_tx_commit(client, raw_tx))?;

    Ok(())
}

pub async fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<()> {
    let res = client.broadcast_tx_commit(raw_tx.encode_to_vec()).await?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}

// TODO: we're assuming here that the app has an auth module which handles this query
fn get_account_latest(address: AccAddress, node: &str) -> Result<QueryAccountResponse> {
    let query = QueryAccountRequest { address };

    run_query::<QueryAccountResponse, RawQueryAccountResponse>(
        query.encode_vec(),
        "/cosmos.auth.v1beta1.Query/Account".into(),
        node,
        None,
    )
}
