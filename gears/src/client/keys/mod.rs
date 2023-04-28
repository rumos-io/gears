use anyhow::{anyhow, Result};
use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command};
use hdpath::{Purpose, StandardHDPath};
use ibc_relayer::keyring::SigningKeyPairSized;
use ibc_relayer::{
    config::AddressType,
    keyring::{Secp256k1KeyPair, SigningKeyPair},
};
use lazy_static::lazy_static;
use std::path::PathBuf;
use text_io::read;

use crate::{client::keys::key_store::DiskStore, utils::get_default_home_dir};

pub mod key_store;

// Values for the HD_PATH copied from
// https://github.com/informalsystems/hermes/blob/d5fa30db6d4a3dcce84435354f3ce4af932c0141/crates/relayer-cli/src/commands/keys/add.rs#L85
lazy_static! {
    static ref HD_PATH: StandardHDPath = StandardHDPath::new(Purpose::Pubkey, 118, 0, 0, 0);
}

pub fn get_keys_command() -> Command {
    Command::new("keys")
        .about("Manage your application's keys")
        .subcommand(get_keys_sub_commands())
        .subcommand_required(true)
}

pub fn get_keys_sub_commands() -> Command {
    Command::new("add")
        .about("Add a private key (either newly generated or recovered) saving it to <name> file")
        .arg(Arg::new("name").required(true))
        .arg(
            Arg::new("overwrite")
                .short('o')
                .long("overwrite")
                .help("Overwrite existing key with same name")
                .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir()
                        .unwrap_or_default()
                        .display()
                        .to_string()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
}

pub fn run_keys_command(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let name = sub_matches
                .get_one::<String>("name")
                .expect("name argument is required preventing None")
                .to_owned();

            let overwrite = sub_matches.get_flag("overwrite");

            let default_home_directory = get_default_home_dir();
            let home = sub_matches
                .get_one::<PathBuf>("home")
                .or(default_home_directory.as_ref())
                .ok_or(anyhow!(
                    "Home argument not provided and OS does not provide a default home directory"
                ))?
                .to_owned();

            let mut key_store = DiskStore::new(home)?;

            check_key_exists(&key_store, &name, overwrite)?;

            println!("> Enter your bip39 mnemonic");
            let mnemonic: String = read!("{}\n");

            let key_pair = Secp256k1KeyPair::from_mnemonic(
                &mnemonic,
                &HD_PATH,
                &AddressType::Cosmos,
                "cosmos",
            )?;

            key_store.add_key(&name, key_pair.clone())?;

            Ok(())
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents None"),
    }
}

fn check_key_exists<S: SigningKeyPairSized>(
    keystore: &DiskStore<S>,
    key_name: &str,
    overwrite: bool,
) -> Result<()> {
    if keystore.get_key(key_name).is_ok() {
        if overwrite {
            println!("key {} will be overwritten", key_name);
            return Ok(());
        } else {
            return Err(anyhow!("A key with name '{key_name}' already exists"));
        }
    }

    Ok(())
}
