use anyhow::{anyhow, Result};
use bip32::Mnemonic;
use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command};
use std::path::PathBuf;
use text_io::read;

use crate::utils::get_default_home_dir;

pub const KEYRING_SUB_DIR: &str = "keyring-file";

pub fn get_keys_command(app_name: &str) -> Command {
    Command::new("keys")
        .about("Manage your application's keys")
        .subcommand(get_keys_sub_commands(app_name))
        .subcommand_required(true)
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
                    get_default_home_dir(app_name)
                        .unwrap_or_default()
                        .display()
                        .to_string()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
}

pub fn run_keys_command(matches: &ArgMatches, app_name: &str) -> Result<()> {
    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let name = sub_matches
                .get_one::<String>("name")
                .expect("name argument is required preventing None")
                .to_owned();

            let recover = sub_matches.get_flag("recover");

            let default_home_directory = get_default_home_dir(app_name);
            let home = sub_matches
                .get_one::<PathBuf>("home")
                .or(default_home_directory.as_ref())
                .ok_or(anyhow!(
                    "Home argument not provided and OS does not provide a default home directory"
                ))?
                .to_owned();
            let keyring_home = home.join(KEYRING_SUB_DIR);

            if recover {
                println!("> Enter your bip39 mnemonic");
                let phrase: String = read!("{}\n");

                let mnemonic = Mnemonic::new(phrase, bip32::Language::English)?;

                keyring::add_key(
                    &name,
                    &mnemonic,
                    keyring::KeyType::Secp256k1,
                    keyring::Backend::File(&keyring_home),
                )?;

                Ok(())
            } else {
                let (mnemonic, key_pair) = keyring::create_key(
                    &name,
                    keyring::KeyType::Secp256k1,
                    keyring::Backend::File(&keyring_home),
                )?;

                println!("Created key {}\nAddress: {}", name, key_pair.get_address());

                println!("\n**Important** write this mnemonic phrase in a safe place.\nIt is the only way to recover your account.\n");
                println!("{}\n", mnemonic.phrase());

                Ok(())
            }
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents None"),
    }
}
