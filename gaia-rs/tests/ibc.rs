use std::{str::FromStr, time::Duration};

use gaia_rs::{client::GaiaTxCommands, GaiaCore};
use gears::client::{
    keys::KeyringBackend,
    tx::{run_tx, TxCommand},
};
use ibc::{
    client::cli::tx::{create::CliCreateClient, IbcCommands, IbcTxCli},
    types::{ConsensusState, Signer},
};
use proto_messages::cosmos::ibc::types::{
    core::{
        client::types::Height, commitment_types::specs::ProofSpecs, host::identifiers::ChainId,
    },
    tendermint::{
        consensus_state::RawConsensusState, AllowUpdate, RawTendermintClientState, TrustThreshold,
    },
};
use serde_json::json;
use tendermint::informal::{trust_threshold::TrustThresholdFraction, Time};
use utilities::{key_add, run_gaia_and_tendermint};

#[path = "./utilities.rs"]
mod utilities;

// https://github.com/cosmos/ibc-go/blob/8f53c21361f9d65448a850c2eafcf3ab3c384a61/modules/light-clients/07-tendermint/types/client_state_test.go#L80
#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn client_create_tx() -> anyhow::Result<()> {
    let (tendermint, _server_thread) = run_gaia_and_tendermint()?;

    key_add(tendermint.1.to_path_buf())?;

    let state = RawTendermintClientState {
        chain_id: ChainId::from_str("test-chain")?,
        trust_level: TrustThreshold::ONE_THIRD,
        trusting_period: Duration::from_secs(2000),
        unbonding_period: Duration::from_secs(2000),
        max_clock_drift: Duration::from_secs(2000),
        latest_height: Height::new(1, 3)?,
        proof_specs: ProofSpecs::cosmos(),
        upgrade_path: Vec::new(),
        allow_update: AllowUpdate {
            after_expiry: true,
            after_misbehaviour: true,
        },
        frozen_height: None,
        verifier: Default::default(), // TODO: How to access this type?
    };

    let consensus = RawConsensusState {
        timestamp: todo!(),
        root: todo!(),
        next_validators_hash: todo!(),
    };

    let cmd = CliCreateClient {
        client_state: serde_json::to_string(&state)?,
        consensus_state: "".to_owned(),
        signer: Signer::from("TODO"),
    };
    let args_cmd = IbcTxCli {
        command: IbcCommands::ClientCreate(cmd),
    };

    let tx_cmd = TxCommand {
        home: tendermint.1.to_path_buf(),
        node: "http://localhost:26657/"
            .try_into()
            .expect("Default should be valid"),
        from_key: "alice".to_owned(),
        chain_id: "test-chain".try_into()?,
        fee: None,
        keyring_backend: KeyringBackend::Test,

        inner: GaiaTxCommands::IBC(args_cmd),
    };

    let _result = run_tx(tx_cmd, &GaiaCore)?;

    Ok(())
}
