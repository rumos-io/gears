use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use auth_crate::Keeper;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use clap_complete::{generate, Generator, Shell};
use database::{Database, RocksDB};
use gears::baseapp::cli::get_run_command;
use gears::client::{init::get_init_command, query::get_query_command, tx::get_tx_command};
use gears::error::AppError;
use gears::types::context_v2::Context;
use gears::x::params::ParamsSubspaceKey;
use human_panic::setup_panic;
use tracing::error;

use gears::{
    baseapp::cli::run_run_command_micro,
    client::{
        init::run_init_command,
        keys::{get_keys_command, run_keys_command},
        query::run_query_command,
        tx::run_tx_command,
    },
};
use ibc_proto::google::protobuf::Any;
use store::StoreKey;
use strum_macros::EnumIter;

use crate::genesis::GenesisState;

mod genesis;
mod module;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn get_completions_command() -> Command {
    Command::new("completions")
        .about("Output shell completions")
        .arg(
            Arg::new("shell")
                .required(true)
                .action(ArgAction::Set)
                .value_parser(value_parser!(Shell)),
        )
}

fn run_completions_command(matches: &ArgMatches) {
    if let Some(generator) = matches.get_one::<Shell>("shell").copied() {
        let mut cmd = build_cli();
        print_completions(generator, &mut cmd);
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn build_cli() -> Command {
    Command::new(APP_NAME)
        .version(env!("GIT_HASH"))
        .subcommand_required(true)
        .subcommand(get_init_command(APP_NAME))
        .subcommand(get_run_command(APP_NAME))
        .subcommand(get_query_command())
        .subcommand(get_keys_command(APP_NAME))
        .subcommand(get_tx_command(APP_NAME))
        .subcommand(get_completions_command())
}

fn main() -> Result<()> {
    setup_panic!();

    //################################
    #[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
    enum GaiaStoreKey {
        Bank,
        Auth,
        Params,
    }

    impl StoreKey for GaiaStoreKey {
        fn name(&self) -> &'static str {
            match self {
                GaiaStoreKey::Bank => "bank",
                GaiaStoreKey::Auth => "acc",
                GaiaStoreKey::Params => "params",
            }
        }
    }

    let params_keeper = gears::x::params::Keeper::new(GaiaStoreKey::Params);
    let bank_keeper =
        bank::Keeper::new(GaiaStoreKey::Bank, params_keeper, GaiaParamsStoreKey::Bank);

    let params_keeper = gears::x::params::Keeper::new(GaiaStoreKey::Params);
    let auth_keeper =
        auth_crate::Keeper::new(GaiaStoreKey::Auth, params_keeper, GaiaParamsStoreKey::Auth);

    #[derive(Debug, Clone)]
    enum Message {
        Bank(bank::Message),
    }

    impl From<Message> for Any {
        fn from(msg: Message) -> Self {
            match msg {
                Message::Bank(msg) => msg.into(),
            }
        }
    }

    impl TryFrom<Any> for Message {
        type Error = proto_messages::Error;

        fn try_from(value: Any) -> Result<Self, Self::Error> {
            if value.type_url.starts_with("/cosmos.bank") {
                Ok(Message::Bank(Any::try_into(value)?))
            } else {
                Err(proto_messages::Error::DecodeGeneral(
                    "message type not recognized".into(),
                ))
            }
        }
    }

    impl proto_messages::cosmos::tx::v1beta1::tx_v2::Message for Message {
        fn get_signers(&self) -> Vec<&proto_types::AccAddress> {
            match self {
                Message::Bank(msg) => msg.get_signers(),
            }
        }

        fn validate_basic(&self) -> std::result::Result<(), String> {
            match self {
                Message::Bank(msg) => msg.validate_basic(),
            }
        }
    }

    //------------------------------------------
    // handler stuff

    #[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone)]
    enum GaiaParamsStoreKey {
        Bank,
        Auth,
        BaseApp,
    }

    impl ParamsSubspaceKey for GaiaParamsStoreKey {
        fn name(&self) -> &'static str {
            match self {
                Self::Bank => "bank/",
                Self::Auth => "auth/",
                Self::BaseApp => "baseapp/",
            }
        }
    }

    #[derive(Debug, Clone)]
    struct Handler {
        bank_handler: bank::Handler<GaiaStoreKey, GaiaParamsStoreKey>,
        auth_handler: auth_crate::Handler<GaiaStoreKey, GaiaParamsStoreKey>,
    }

    impl Handler {
        pub fn new() -> Handler {
            let params_keeper = gears::x::params::Keeper::new(GaiaStoreKey::Params);
            let bank_keeper =
                bank::Keeper::new(GaiaStoreKey::Bank, params_keeper, GaiaParamsStoreKey::Bank);

            let params_keeper = gears::x::params::Keeper::new(GaiaStoreKey::Params);
            let auth_keeper = auth_crate::Keeper::new(
                GaiaStoreKey::Bank,
                params_keeper,
                GaiaParamsStoreKey::Bank,
            );
            Handler {
                bank_handler: bank::Handler::new(bank_keeper),
                auth_handler: auth_crate::Handler::new(auth_keeper),
            }
        }
    }

    impl gears::baseapp::Handler<Message, GaiaStoreKey> for Handler {
        fn handle<DB: Database>(
            &self,
            ctx: &mut Context<DB, GaiaStoreKey>,
            msg: &Message,
        ) -> Result<(), AppError> {
            match msg {
                Message::Bank(msg) => self.bank_handler.handle(ctx, msg),
            }
        }

        fn init_genesis<DB: Database>(
            &self,
            ctx: &mut Context<DB, GaiaStoreKey>,
            raw: bytes::Bytes,
        ) {
            let genesis = String::from_utf8(raw.into())
                .map_err(|e| AppError::Genesis(e.to_string()))
                .and_then(|f| GenesisState::from_str(&f))
                .unwrap_or_else(|e| {
                    error!(
                        "Invalid genesis provided by Tendermint.\n{}\nTerminating process",
                        e.to_string()
                    );
                    std::process::exit(1)
                });

            self.bank_handler.init_genesis(ctx, genesis.bank);
            self.auth_handler.init_genesis(ctx, genesis.auth);
        }
    }

    let handler = Handler::new(); //TODO: crete directly

    //---------------------------------------------

    // #[derive(Debug, Clone)]
    // struct Message {}

    // impl gears::baseapp::Message for Message {
    //     fn get_signers(&self) -> Vec<&proto_types::AccAddress> {
    //         return vec![];
    //     }

    //     fn validate_basic(&self) -> std::result::Result<(), String> {
    //         return Ok(());
    //     }
    // }

    // #[derive(Debug, Clone)]
    // struct Decoder {}

    // impl gears::baseapp::Decoder<Message> for Decoder {
    //     fn decode(raw: Vec<u8>) -> Message {
    //         return Message {};
    //     }
    // };

    let params_keeper = gears::x::params::Keeper::new(GaiaStoreKey::Params);

    //################################

    let cli = build_cli();
    let matches = cli.get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => run_init_command(sub_matches, APP_NAME),
        Some(("run", sub_matches)) => run_run_command_micro::<
            GaiaStoreKey,
            GaiaParamsStoreKey,
            Message,
            bank::Keeper<GaiaStoreKey, GaiaParamsStoreKey>,
            auth_crate::Keeper<GaiaStoreKey, GaiaParamsStoreKey>,
            Handler,
        >(
            sub_matches,
            APP_NAME,
            bank_keeper,
            auth_keeper,
            params_keeper,
            GaiaParamsStoreKey::BaseApp,
            handler,
        ),
        Some(("query", sub_matches)) => run_query_command(sub_matches)?,
        Some(("keys", sub_matches)) => run_keys_command(sub_matches, APP_NAME)?,
        Some(("tx", sub_matches)) => run_tx_command(sub_matches, APP_NAME)?,
        Some(("completions", sub_matches)) => run_completions_command(sub_matches),
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    Ok(())
}
