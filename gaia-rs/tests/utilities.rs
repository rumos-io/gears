//! This modules should be added to test modules with `#[path = "./utilities.rs"]` as it contains gaia specific code and dedicated crate is bothersome.
#![allow(dead_code)]

use std::{path::PathBuf, str::FromStr, sync::OnceLock, time::Duration};

use gaia_rs::{
    abci_handler::GaiaABCIHandler, config::AppConfig, genesis::GenesisState,
    store_keys::GaiaParamsStoreKey, GaiaApplication, GaiaCore,
};
use gears::{
    application::node::NodeApplication,
    baseapp::genesis::{Genesis, GenesisError},
    commands::{
        client::keys::{keys, AddKeyCommand, KeyCommand, KeyringBackend},
        node::{
            run::{LogLevel, RunCommand},
            AppCommands,
        },
    },
    config::{DEFAULT_ADDRESS, DEFAULT_GRPC_LISTEN_ADDR, DEFAULT_REST_LISTEN_ADDR},
    store::database::{sled::SledDb, DBBuilder},
    types::{
        base::{coin::UnsignedCoin, coins::UnsignedCoins},
        denom::Denom,
    },
};
use gears::{
    types::address::AccAddress,
    utils::tendermint::{TempDir, TendermintSubprocess},
};

pub const TENDERMINT_PATH: &str = "./tests/assets";
pub const BIP39_MNEMONIC : &str = "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow";

pub const NODE_URL_STR: &str = "http://localhost:26657/";

pub fn node_url() -> url::Url {
    NODE_URL_STR.try_into().expect("Default should be valid")
}

pub const ACC_ADDRESS: &str = "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux";

pub fn acc_address() -> AccAddress {
    AccAddress::from_bech32(ACC_ADDRESS).expect("Default Address should be valid")
}

pub fn tendermint() -> &'static TendermintSubprocess {
    static TENDERMINT: OnceLock<(TendermintSubprocess, std::thread::JoinHandle<()>)> =
        OnceLock::new();

    &TENDERMINT
        .get_or_init(|| {
            let res = run_gaia_and_tendermint([(acc_address(), default_coin(200_000_000_u32))]);

            match res {
                Ok(res) => res,
                Err(err) => panic!("Failed to start tendermint with err: {err}"),
            }
        })
        .0
}

/// Helper method to start gaia node and tendermint in tmp folder
pub fn run_gaia_and_tendermint(
    accounts: impl IntoIterator<Item = (AccAddress, UnsignedCoin)>,
) -> anyhow::Result<(TendermintSubprocess, std::thread::JoinHandle<()>)> {
    let tmp_dir = TempDir::new()?;
    let tmp_path = tmp_dir.to_path_buf();

    key_add(tmp_dir.to_path_buf(), KEY_NAME, BIP39_MNEMONIC)?;

    let genesis = {
        let mut genesis = MockGenesis::default();

        for (acc, coin) in accounts {
            genesis.add_genesis_account(acc, UnsignedCoins::new([coin])?)?;
        }

        genesis
    };

    let tendermint =
        TendermintSubprocess::run_tendermint::<_, AppConfig>(tmp_dir, TENDERMINT_PATH, &genesis)?;

    std::thread::sleep(Duration::from_secs(10));

    let server_thread = std::thread::spawn(move || {
        let node = NodeApplication::<GaiaCore, SledDb, _, _>::new(
            GaiaCore,
            DBBuilder,
            GaiaABCIHandler::new,
            GaiaParamsStoreKey::BaseApp,
        );

        let cmd = RunCommand {
            home: tmp_path,
            address: Some(DEFAULT_ADDRESS),
            rest_listen_addr: Some(DEFAULT_REST_LISTEN_ADDR),
            grpc_listen_addr: Some(DEFAULT_GRPC_LISTEN_ADDR),
            read_buf_size: 1048576,
            log_level: LogLevel::Off,
            min_gas_prices: Default::default(),
        };

        let _ = node.execute::<GaiaApplication>(AppCommands::Run(cmd));
    });

    std::thread::sleep(Duration::from_secs(10));

    Ok((tendermint, server_thread))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct MockGenesis(pub GenesisState);

impl Genesis for MockGenesis {
    fn add_genesis_account(
        &mut self,
        address: AccAddress,
        coins: UnsignedCoins,
    ) -> Result<(), GenesisError> {
        self.0.add_genesis_account(address, coins)
    }
}

pub const KEY_NAME: &str = "alice";

pub fn key_add(home: impl Into<PathBuf>, name: &str, mnemonic: &str) -> anyhow::Result<()> {
    let cmd = AddKeyCommand {
        name: name.to_owned(),
        recover: true,
        home: home.into(),
        keyring_backend: KeyringBackend::Test,
        bip39_mnemonic: Some(mnemonic.to_owned()),
    };

    keys(KeyCommand::Add(cmd))?;

    Ok(())
}

pub fn default_coin(amount: u32) -> UnsignedCoin {
    UnsignedCoin {
        denom: Denom::from_str("uatom").expect("default denom should be valid"),
        amount: amount.into(),
    }
}
