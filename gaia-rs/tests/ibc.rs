use std::{str::FromStr, time::Duration};

use gaia_rs::{client::GaiaTxCommands, GaiaCoreClient};
use gears::client::{
    keys::KeyringBackend,
    tx::{run_tx, TxCommand},
};
use ibc::{
    client::cli::tx::{
        create::CliCreateClient, update::CliUpdateClient, upgrade::CliUpgradeClient, IbcCommands,
        IbcTxCli,
    },
    types::Signer,
};
use proto_messages::cosmos::ibc::types::{
    core::{
        client::types::Height,
        commitment::CommitmentRoot,
        commitment_types::specs::ProofSpecs,
        host::identifiers::{ChainId, ClientId},
    },
    tendermint::{
        consensus_state::RawConsensusState, AllowUpdate, RawTendermintClientState, TrustThreshold,
        WrappedTendermintClientState,
    },
};
use tendermint_tmp::{Hash, Time};
use utilities::{key_add, node_url, run_gaia_and_tendermint, KEY_NAME};

#[path = "./utilities.rs"]
mod utilities;

// https://github.com/cosmos/ibc-go/blob/8f53c21361f9d65448a850c2eafcf3ab3c384a61/modules/light-clients/07-tendermint/types/client_state_test.go#L80
#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn client_create_tx() -> anyhow::Result<()> {
    let (tendermint, _server_thread) = run_gaia_and_tendermint()?;

    key_add(tendermint.1.to_path_buf())?;

    let state = RawTendermintClientState::new(
        ChainId::from_str("test-1")?,
        TrustThreshold::ONE_THIRD,
        Duration::from_secs(1000),
        Duration::from_secs(2000),
        Duration::from_secs(2000),
        Height::new(1, 3)?,
        ProofSpecs::cosmos(),
        Vec::new(),
        AllowUpdate {
            after_expiry: true,
            after_misbehaviour: true,
        },
    )?;

    let state = WrappedTendermintClientState::from(state);

    let consensus = RawConsensusState {
        timestamp: Time::now(),
        root: CommitmentRoot::from(Vec::new()),
        next_validators_hash: Hash::None,
    };

    let cmd = CliCreateClient {
        client_state: serde_json::to_string(&state)?,
        consensus_state: serde_json::to_string(&consensus)?,
        signer: Signer::from("TODO"),
    };
    let args_cmd = IbcTxCli {
        command: IbcCommands::ClientCreate(cmd),
    };

    let tx_cmd = TxCommand {
        home: tendermint.1.to_path_buf(),
        node: node_url(),
        from_key: KEY_NAME.to_owned(),
        chain_id: "test-chain".try_into()?,
        fee: None,
        keyring_backend: KeyringBackend::Test,

        inner: GaiaTxCommands::IBC(args_cmd),
    };

    let _result = run_tx(tx_cmd, &GaiaCoreClient)?;

    Ok(())
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn client_update_tx() -> anyhow::Result<()> {
    let (tendermint, _server_thread) = run_gaia_and_tendermint()?;
    key_add(tendermint.1.to_path_buf())?;

    let cli_args = CliUpdateClient {
        client_id: ClientId::from_str("07-tendermint-0")?,
        client_message: String::new(), // TODO: !!!!!! what msg here should be?
        signer: Signer::from("TODO"),
    };

    let args_cmd = IbcTxCli {
        command: IbcCommands::ClientUpdate(cli_args),
    };

    let tx_cmd = TxCommand {
        home: tendermint.1.to_path_buf(),
        node: node_url(),
        from_key: KEY_NAME.to_owned(),
        chain_id: "test-chain".try_into()?,
        fee: None,
        keyring_backend: KeyringBackend::Test,

        inner: GaiaTxCommands::IBC(args_cmd),
    };

    let _result = run_tx(tx_cmd, &GaiaCoreClient)?;

    Ok(())
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn client_upgrade_tx() -> anyhow::Result<()> {
    let (tendermint, _server_thread) = run_gaia_and_tendermint()?;
    key_add(tendermint.1.to_path_buf())?;

    let cli_args = CliUpgradeClient {
        client_id: ClientId::from_str("07-tendermint-0")?,
        upgraded_client_state: String::new(),
        upgraded_consensus_state: String::new(),
        proof_upgrade_client: String::new(),
        proof_upgrade_consensus_state: String::new(),
        signer: Signer::from("TODO"),
    };

    let args_cmd = IbcTxCli {
        command: IbcCommands::ClientUpgrade(cli_args),
    };

    let tx_cmd = TxCommand {
        home: tendermint.1.to_path_buf(),
        node: node_url(),
        from_key: KEY_NAME.to_owned(),
        chain_id: "test-chain".try_into()?,
        fee: None,
        keyring_backend: KeyringBackend::Test,

        inner: GaiaTxCommands::IBC(args_cmd),
    };

    let _result = run_tx(tx_cmd, &GaiaCoreClient)?;

    Ok(())
}
