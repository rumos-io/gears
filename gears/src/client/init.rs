use std::{fs, path::PathBuf};

use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command};
use serde::Serialize;
use tendermint_informal::chain::Id;

use crate::utils::{get_default_home_dir, get_genesis_file_from_home_dir};

pub fn get_init_command(app_name: &str) -> Command {
    Command::new("init")
        .about("Initialize configuration files")
        .arg(Arg::new("moniker").required(true))
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
        .arg(
            Arg::new("chain-id")
                .long("chain-id")
                .help("Genesis file chain-id")
                .default_value("test-chain")
                .action(ArgAction::Set)
                .value_parser(value_parser!(Id)),
        )
}

pub fn run_init_command<G: Serialize>(
    sub_matches: &ArgMatches,
    app_name: &str,
    app_genesis_state: G,
) {
    let moniker = sub_matches
        .get_one::<String>("moniker")
        .expect("moniker argument is required preventing `None`");

    let default_home_directory = get_default_home_dir(app_name);

    let home = sub_matches
        .get_one::<PathBuf>("home")
        .or(default_home_directory.as_ref())
        .unwrap_or_else(|| {
            println!("Home argument not provided and OS does not provide a default home directory");
            std::process::exit(1)
        });

    let chain_id = sub_matches
        .get_one::<Id>("chain-id")
        .expect("has a default value so will never be None")
        .clone();

    // Create config directory
    let mut config_dir = home.clone();
    config_dir.push("config");
    fs::create_dir_all(&config_dir).unwrap_or_else(|e| {
        println!("Could not create config directory {}", e);
        std::process::exit(1)
    });

    // Create data directory
    let mut data_dir = home.clone();
    data_dir.push("data");
    fs::create_dir_all(&data_dir).unwrap_or_else(|e| {
        println!("Could not create data directory {}", e);
        std::process::exit(1)
    });

    // Write tendermint config file
    let mut tm_config_file_path = config_dir.clone();
    tm_config_file_path.push("config.toml");
    let tm_config_file = std::fs::File::create(&tm_config_file_path).unwrap_or_else(|e| {
        println!("Could not create config file {}", e);
        std::process::exit(1)
    });
    tendermint::write_tm_config(tm_config_file, moniker).unwrap_or_else(|e| {
        println!("Error writing config file {}", e);
        std::process::exit(1)
    });
    println!(
        "Tendermint config written to {}",
        tm_config_file_path.display()
    );

    // Create node key file
    let mut node_key_file_path = config_dir.clone();
    node_key_file_path.push("node_key.json");
    let node_key_file = std::fs::File::create(&node_key_file_path).unwrap_or_else(|e| {
        println!("Could not create node key file {}", e);
        std::process::exit(1)
    });

    // Create private validator key file
    let mut priv_validator_key_file_path = config_dir.clone();
    priv_validator_key_file_path.push("priv_validator_key.json");
    let priv_validator_key_file = std::fs::File::create(&priv_validator_key_file_path)
        .unwrap_or_else(|e| {
            println!("Could not create private validator key file {}", e);
            std::process::exit(1)
        });

    let app_state = serde_json::to_value(app_genesis_state).unwrap();

    // Create genesis file
    let mut genesis_file_path = home.clone();
    get_genesis_file_from_home_dir(&mut genesis_file_path);
    let genesis_file = std::fs::File::create(&genesis_file_path).unwrap_or_else(|e| {
        println!("Could not create genesis file {}", e);
        std::process::exit(1)
    });

    // Write key and genesis
    tendermint::write_keys_and_genesis(
        node_key_file,
        priv_validator_key_file,
        genesis_file,
        app_state,
        chain_id,
    )
    .unwrap_or_else(|e| {
        println!("Error writing key and genesis files {}", e);
        std::process::exit(1)
    });
    println!(
        "Key files written to {} and {}",
        node_key_file_path.display(),
        priv_validator_key_file_path.display()
    );
    println!("Genesis file written to {}", genesis_file_path.display(),);

    // Write write private validator state file
    let mut state_file_path = data_dir.clone();
    state_file_path.push("priv_validator_state.json");
    let state_file = std::fs::File::create(&state_file_path).unwrap_or_else(|e| {
        println!("Could not create private validator state file {}", e);
        std::process::exit(1)
    });
    tendermint::write_priv_validator_state(state_file).unwrap_or_else(|e| {
        println!("Error writing private validator state file {}", e);
        std::process::exit(1)
    });
    println!(
        "Private validator state written to {}",
        state_file_path.display()
    );
}
