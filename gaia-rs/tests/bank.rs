use std::time::Duration;

use bank::cli::query::{BalancesCommand, BankCommands, BankQueryCli};
use gaia_rs::{
    abci_handler::ABCIHandler,
    config::AppConfig,
    genesis::GenesisState,
    store_keys::{GaiaParamsStoreKey, GaiaStoreKey},
    GaiaApplication, GaiaCore,
};
use gears::{
    application::{command::app::AppCommands, node::NodeApplication},
    baseapp::{run::RunCommand, Genesis},
    client::query::{run_query, QueryCommand},
    config::{DEFAULT_ADDRESS, DEFAULT_REST_LISTEN_ADDR, DEFAULT_TENDERMINT_RPC_ADDRESS},
};
use proto_types::AccAddress;
use utils::testing::TmpChild;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct MockGenesis(pub GenesisState);

impl Genesis for MockGenesis {
    fn add_genesis_account(
        &mut self,
        address: proto_types::AccAddress,
        coins: proto_messages::cosmos::base::v1beta1::SendCoins,
    ) -> Result<(), gears::error::AppError> {
        self.0.add_genesis_account(address, coins)
    }
}

const TENDERMINT_PATH: &str = "./tests/assets";

#[test]
fn balances_query() -> anyhow::Result<()> {
    let tendermint =
        TmpChild::start_tendermint::<_, AppConfig>(TENDERMINT_PATH, &MockGenesis::default())?;

    let _server_thread = std::thread::spawn(move || {
        let node = NodeApplication::<'_, GaiaCore, GaiaApplication>::new(
            GaiaCore,
            &ABCIHandler::new,
            GaiaStoreKey::Params,
            GaiaParamsStoreKey::BaseApp,
        );

        let cmd = RunCommand {
            home: tendermint.1.to_path_buf(),
            address: DEFAULT_ADDRESS,
            rest_listen_addr: DEFAULT_REST_LISTEN_ADDR,
            read_buf_size: 1048576,
            log_level: gears::baseapp::run::LogLevel::Off,
        };

        let _ = node.execute(AppCommands::Run(cmd));
    });

    std::thread::sleep(Duration::from_secs(2));

    let query = BalancesCommand {
        address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")?,
    };

    let result = run_query(
        QueryCommand {
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            height: None,
            inner: gaia_rs::client::GaiaQueryCommands::Bank(BankQueryCli {
                command: BankCommands::Balances(query),
            }),
        },
        &GaiaCore,
    )?;

    let expected = r#"{
            "balances": [
              {
                "denom": "uatom",
                "amount": "34"
              }
            ],
            "pagination": null
          }"#;

    assert_eq!(&result, expected);

    Ok(())
}

#[test]
fn denom_query() -> anyhow::Result<()> {
    let tendermint =
        TmpChild::start_tendermint::<_, AppConfig>(TENDERMINT_PATH, &MockGenesis::default())?;

    let _server_thread = std::thread::spawn(move || {
        let node = NodeApplication::<'_, GaiaCore, GaiaApplication>::new(
            GaiaCore,
            &ABCIHandler::new,
            GaiaStoreKey::Params,
            GaiaParamsStoreKey::BaseApp,
        );

        let cmd = RunCommand {
            home: tendermint.1.to_path_buf(),
            address: DEFAULT_ADDRESS,
            rest_listen_addr: DEFAULT_REST_LISTEN_ADDR,
            read_buf_size: 1048576,
            log_level: gears::baseapp::run::LogLevel::Off,
        };

        let _ = node.execute(AppCommands::Run(cmd));
    });

    std::thread::sleep(Duration::from_secs(2));

    let result = run_query(
        QueryCommand {
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            height: None,
            inner: gaia_rs::client::GaiaQueryCommands::Bank(BankQueryCli {
                command: BankCommands::DenomMetadata,
            }),
        },
        &GaiaCore,
    )?;

    let expected = r#""#;

    assert_eq!(&result, expected);

    Ok(())
}
