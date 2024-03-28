use std::time::Duration;

use auth::cli::query::{AccountCommand, AuthCommands, AuthQueryCli, AuthQueryResponse};
use gaia_rs::{
    abci_handler::ABCIHandler,
    client::GaiaQueryCommands,
    config::AppConfig,
    genesis::GenesisState,
    query::GaiaQueryResponse,
    store_keys::{GaiaParamsStoreKey, GaiaStoreKey},
    GaiaApplication, GaiaCore, GaiaCoreClient,
};
use gears::{
    application::{command::app::AppCommands, node::NodeApplication},
    baseapp::{run::RunCommand, Genesis},
    client::query::{run_query, QueryCommand},
    config::{DEFAULT_ADDRESS, DEFAULT_REST_LISTEN_ADDR, DEFAULT_TENDERMINT_RPC_ADDRESS},
};
use proto_messages::cosmos::auth::v1beta1::{Account, BaseAccount, QueryAccountResponse};
use proto_types::AccAddress;
use utils::testing::{TempDir, TmpChild};

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
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn account_query() -> anyhow::Result<()> {
    let (_tendermint, _server_thread) = run_gaia_and_tendermint()?;

    let acc_adress = AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
        .expect("Valid value");

    let query = AccountCommand {
        address: acc_adress.clone(),
    };

    let cmd = QueryCommand {
        node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
        height: None,
        inner: GaiaQueryCommands::Auth(AuthQueryCli {
            command: AuthCommands::Account(query),
        }),
    };

    let result = run_query(cmd, &GaiaCoreClient)?;

    let expected = GaiaQueryResponse::Auth(AuthQueryResponse::Account(QueryAccountResponse {
        account: Account::Base(BaseAccount {
            address: acc_adress,
            pub_key: None,
            account_number: 0,
            sequence: 0,
        }),
    }));

    assert_eq!(result, expected);

    Ok(())
}

/// Helper method to start gaia node and tendermint in tmp folder
fn run_gaia_and_tendermint() -> anyhow::Result<(TmpChild, std::thread::JoinHandle<()>)> {
    let tmp_dir = TempDir::new()?;
    let tmp_path = tmp_dir.to_path_buf();

    let tendermint = TmpChild::run_tendermint::<_, AppConfig>(
        tmp_dir,
        TENDERMINT_PATH,
        &MockGenesis::default(),
    )?;

    std::thread::sleep(Duration::from_secs(10));

    let server_thread = std::thread::spawn(move || {
        let node = NodeApplication::<'_, GaiaCore, GaiaApplication>::new(
            GaiaCore,
            &ABCIHandler::new,
            GaiaStoreKey::Params,
            GaiaParamsStoreKey::BaseApp,
        );

        let cmd = RunCommand {
            home: tmp_path,
            address: DEFAULT_ADDRESS,
            rest_listen_addr: DEFAULT_REST_LISTEN_ADDR,
            read_buf_size: 1048576,
            log_level: gears::baseapp::run::LogLevel::Off,
        };

        let _ = node.execute(AppCommands::Run(cmd));
    });

    std::thread::sleep(Duration::from_secs(10));

    Ok((tendermint, server_thread))
}
