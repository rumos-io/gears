use anyhow::Result;
use bip32::Mnemonic;
use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command, ValueEnum};
use std::path::PathBuf;
use text_io::read;

use crate::utils::{self, get_default_home_dir};

const KEYRING_SUB_DIR_FILE: &str = "keyring-file";
const KEYRING_SUB_DIR_TEST: &str = "keyring-test";

pub fn get_keys_command(app_name: &str) -> Command {
    Command::new("keys")
        .about("Manage your application's keys")
        .subcommand(get_keys_sub_commands(app_name))
        .subcommand_required(true)
}

#[derive(ValueEnum, Clone, Default, Debug)]
pub enum KeyringBackend {
    #[default]
    File,
    Test,
}

impl KeyringBackend {
    pub fn get_sub_dir(&self) -> &str {
        match self {
            KeyringBackend::File => KEYRING_SUB_DIR_FILE,
            KeyringBackend::Test => KEYRING_SUB_DIR_TEST,
        }
    }

    pub fn to_keyring_backend<'a>(&self, path: &'a PathBuf) -> keyring::Backend<'a> {
        match self {
            KeyringBackend::File => keyring::Backend::File(&path),
            KeyringBackend::Test => keyring::Backend::Test(&path),
        }
    }
}

pub fn get_keys_sub_commands(app_name: &str) -> Command {
    Command::new("add")
        .about("Add a private key (either newly generated or recovered) saving it to <name> file")
        .arg(Arg::new("name").required(true))
        .arg(
            Arg::new("recover")
                .short('r')
                .long("recover")
                .help("Provide seed phrase to recover existing key instead of creating")
                .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir(app_name).unwrap_or_default().display()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("keyring-backend")
                .long("keyring-backend")
                .help("Select keyring's backend (file|test) (default \"file\")")
                .action(ArgAction::Set)
                .value_parser(value_parser!(KeyringBackend)),
        )
}

#[derive(Debug, Clone)]
pub enum KeyCommand {
    Add {
        name: String,
        recover: bool,
        home: PathBuf,
        keyring_backend: KeyringBackend,
    },
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct KeyCommandParseError(pub String);
impl TryFrom<&ArgMatches> for KeyCommand {
    type Error = KeyCommandParseError;

    fn try_from(value: &ArgMatches) -> Result<Self, Self::Error> {
        match value.subcommand() {
            Some(("add", sub_matches)) => {
                let name = sub_matches
                    .get_one::<String>("name")
                    .ok_or(KeyCommandParseError(
                        "name argument is required preventing None".to_owned(),
                    ))?
                    .to_owned();

                let recover = sub_matches.get_flag("recover");

                let home = sub_matches
                    .get_one::<PathBuf>("home")
                    .cloned()
                    .or(utils::default_home())
                    .ok_or(KeyCommandParseError(
                        "Home argument not provided and OS does not provide a default home directory".to_owned()
                    ))?
                    .to_owned();

                let keyring_backend = sub_matches
                    .get_one::<KeyringBackend>("keyring-backend")
                    .cloned()
                    .unwrap_or_default();

                Ok(Self::Add {
                    name,
                    recover,
                    home,
                    keyring_backend,
                })
            }
            _ => Err(KeyCommandParseError(
                "exhausted list of subcommands and subcommand_required prevents None".to_owned(),
            )),
        }
    }
}

pub fn keys(command: KeyCommand) -> Result<()> {
    match command {
        KeyCommand::Add {
            name,
            recover,
            home,
            keyring_backend,
        } => {
            let keyring_home = home.join(keyring_backend.get_sub_dir());

            let backend = keyring_backend.to_keyring_backend(&keyring_home);

            if recover {
                println!("> Enter your bip39 mnemonic");
                let phrase: String = read!("{}\n");

                let mnemonic = Mnemonic::new(phrase, bip32::Language::English)?;

                keyring::add_key(&name, &mnemonic, keyring::KeyType::Secp256k1, backend)?;

                Ok(())
            } else {
                let (mnemonic, key_pair) =
                    keyring::create_key(&name, keyring::KeyType::Secp256k1, backend)?;

                println!("Created key {}\nAddress: {}", name, key_pair.get_address());

                println!("\n**Important** write this mnemonic phrase in a safe place.\nIt is the only way to recover your account.\n");
                println!("{}", mnemonic.phrase());

                Ok(())
            }
        }
    }
}
