use std::{str::FromStr, time::Duration};

use bank::cli::{
    query::{BalancesCommand, BankCommands as BankQueryCommands, BankQueryCli, BankQueryResponse},
    tx::{BankCommands, BankTxCli},
};
use gaia_rs::{
    abci_handler::ABCIHandler,
    client::GaiaTxCommands,
    genesis::GenesisState,
    query::GaiaQueryResponse,
    store_keys::{GaiaParamsStoreKey, GaiaStoreKey},
    GaiaApplication, GaiaCore,
};
use gears::{
    application::{command::app::AppCommands, node::NodeApplication},
    baseapp::{run::RunCommand, Genesis},
    client::{
        keys::KeyringBackend,
        query::{run_query, QueryCommand},
        tx::{run_tx, TxCommand},
    },
    config::{DEFAULT_ADDRESS, DEFAULT_REST_LISTEN_ADDR, DEFAULT_TENDERMINT_RPC_ADDRESS},
};
use proto_messages::cosmos::{
    bank::v1beta1::{QueryAllBalancesResponse, QueryDenomsMetadataResponse},
    base::v1beta1::Coin,
};
use proto_types::{AccAddress, Denom};
use tendermint::informal::chain::Id;
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
    let tendermint = TmpChild::start_tendermint(TENDERMINT_PATH)?;

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
                command: BankQueryCommands::Balances(query),
            }),
        },
        &GaiaCore,
    )?;

    let expected = GaiaQueryResponse::Bank(bank::cli::query::BankQueryResponse::Balances(
        QueryAllBalancesResponse {
            balances: vec![Coin {
                denom: Denom::from_str("uatom")?,
                amount: 34_u32.into(),
            }],
            pagination: None,
        },
    ));

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn denom_query() -> anyhow::Result<()> {
    let tendermint = TmpChild::start_tendermint(TENDERMINT_PATH)?;

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
                command: BankQueryCommands::DenomMetadata,
            }),
        },
        &GaiaCore,
    )?;

    let expected = GaiaQueryResponse::Bank(BankQueryResponse::DenomMetadata(
        QueryDenomsMetadataResponse {
            metadatas: Vec::new(),
            pagination: None,
        },
    ));

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn send_tx() -> anyhow::Result<()> {
    let tendermint = TmpChild::start_tendermint(TENDERMINT_PATH)?;

    let home = tendermint.1.to_path_buf();

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

    let tx_cmd = BankCommands::Send {
        to_address: AccAddress::from_bech32("cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut")?,
        amount: Coin::from_str("10uatom")?,
    };

    let _result = run_tx(
        TxCommand {
            home,
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            from_key: "alice".to_owned(),
            chain_id: Id::try_from("test-chain")?,
            fee: None,
            keyring_backend: KeyringBackend::File,
            inner: GaiaTxCommands::Bank(BankTxCli { command: tx_cmd }),
        },
        &GaiaCore,
    )?;

    Ok(())
}
