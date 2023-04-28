use std::{fs, path::PathBuf};

use clap::{arg, value_parser, Arg, ArgAction, ArgMatches, Command};

use crate::{types::GenesisState, utils::get_default_home_dir};

pub fn get_init_command() -> Command {
    Command::new("init")
        .about("Initialize configuration files")
        .arg(Arg::new("moniker").required(true))
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
        .arg(
            arg!(--id)
                .help("Genesis file chain-id")
                .default_value("test-chain")
                .action(ArgAction::Set),
        )
}

pub fn run_init_command(sub_matches: &ArgMatches) {
    let moniker = sub_matches
        .get_one::<String>("moniker")
        .expect("moniker argument is required preventing `None`");

    let default_home_directory = get_default_home_dir();

    let home = sub_matches
        .get_one::<PathBuf>("home")
        .or(default_home_directory.as_ref())
        .unwrap_or_else(|| {
            println!("Home argument not provided and OS does not provide a default home directory");
            std::process::exit(1)
        });

    let _chain_id = sub_matches
        .get_one::<String>("id")
        .expect("has a default value so will never be None");

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

    // Build genesis state
    // TODO: get defaults from the modules
    let app_state = GenesisState {
        bank: crate::x::bank::GenesisState {
            balances: vec![crate::x::bank::Balance {
                address: proto_types::AccAddress::from_bech32(
                    "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                )
                .unwrap(),
                coins: vec![proto_messages::cosmos::base::v1beta1::Coin {
                    denom: proto_types::Denom::try_from(String::from("uatom")).unwrap(),
                    amount: cosmwasm_std::Uint256::from_u128(34),
                }],
            }],
            params: crate::x::bank::Params {
                default_send_enabled: true,
            },
        },
        auth: crate::x::auth::GenesisState {
            accounts: vec![proto_messages::cosmos::auth::v1beta1::BaseAccount {
                address: proto_types::AccAddress::from_bech32(
                    "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                )
                .unwrap(),
                pub_key: None,
                account_number: 0,
                sequence: 0,
            }],
            params: crate::x::auth::Params {
                max_memo_characters: 256,
                tx_sig_limit: 7,
                tx_size_cost_per_byte: 10,
                sig_verify_cost_ed25519: 590,
                sig_verify_cost_secp256k1: 1000,
            },
        },
    };
    let app_state = serde_json::to_value(app_state).unwrap();

    // Create genesis file
    let mut genesis_file_path = config_dir.clone();
    genesis_file_path.push("genesis.json");
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
