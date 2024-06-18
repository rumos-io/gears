use crate::utilities::{key_add, run_gaia_and_tendermint, KEY_NAME};
use bank::cli::tx::{BankCommands, BankTxCli};
use gaia_rs::{
    client::{GaiaTxCommands, WrappedGaiaTxCommands},
    GaiaCoreClient,
};
use gears::{
    commands::client::{
        keys::KeyringBackend,
        tx::{run_tx, Keyring, LocalInfo, TxCommand},
    },
    config::DEFAULT_TENDERMINT_RPC_ADDRESS,
    tendermint::{rpc::response::tx::broadcast::Response, types::chain_id::ChainId},
    types::{
        address::{AccAddress, ValAddress},
        base::coin::Coin,
        uint::Uint256,
    },
};
use staking::cli::tx::{StakingCommands, StakingTxCli};
use std::str::FromStr;

#[path = "./utilities.rs"]
mod utilities;

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn create_validator() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}".to_string();
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(100u64),
    };
    let tx_cmd = StakingCommands::CreateValidator {
        pubkey,
        amount,
        moniker: "test".to_string(),
        identity: "".to_string(),
        website: "".to_string(),
        security_contact: "".to_string(),
        details: "".to_string(),
        commission_rate: "0.1".to_string(),
        commission_max_rate: "0.2".to_string(),
        commission_max_change_rate: "0.01".to_string(),
        min_self_delegation: Uint256::one(),
    };

    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;

    assert!(check_tx.code.is_ok());
    assert_eq!(check_tx.events.len(), 0);
    assert!(deliver_tx.code.is_ok());
    assert_eq!(deliver_tx.events.len(), 4);
    assert!(deliver_tx
        .events
        .iter()
        .any(|e| e.kind == "create_validator"));
    assert!(deliver_tx.events.iter().any(|e| e.kind == "coin_spent"));
    assert!(deliver_tx.events.iter().any(|e| e.kind == "coin_received"));

    Ok(())
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn delegate() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}".to_string();
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(100u64),
    };
    let tx_cmd = StakingCommands::CreateValidator {
        pubkey,
        amount,
        moniker: "test".to_string(),
        identity: "".to_string(),
        website: "".to_string(),
        security_contact: "".to_string(),
        details: "".to_string(),
        commission_rate: "0.1".to_string(),
        commission_max_rate: "0.2".to_string(),
        commission_max_change_rate: "0.01".to_string(),
        min_self_delegation: Uint256::one(),
    };

    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(check_tx.code.is_ok());
    assert!(deliver_tx.code.is_ok());

    /* */

    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    let tx_cmd = StakingCommands::Delegate {
        validator_address: ValAddress::from_bech32(
            "cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4",
        )?,
        amount,
    };
    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;

    assert!(check_tx.code.is_ok());
    assert_eq!(check_tx.events.len(), 0);
    assert!(deliver_tx.code.is_ok());
    assert_eq!(deliver_tx.events.len(), 4);
    assert!(deliver_tx.events.iter().any(|e| e.kind == "delegate"));
    assert!(deliver_tx.events.iter().any(|e| e.kind == "coin_spent"));
    assert!(deliver_tx.events.iter().any(|e| e.kind == "coin_received"));

    Ok(())
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn redelegate() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;

    // create source validator
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}".to_string();
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(100u64),
    };
    let tx_cmd = StakingCommands::CreateValidator {
        pubkey,
        amount,
        moniker: "test".to_string(),
        identity: "".to_string(),
        website: "".to_string(),
        security_contact: "".to_string(),
        details: "".to_string(),
        commission_rate: "0.1".to_string(),
        commission_max_rate: "0.2".to_string(),
        commission_max_change_rate: "0.01".to_string(),
        min_self_delegation: Uint256::one(),
    };

    let Response {
        check_tx: _,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(deliver_tx.code.is_ok());

    // send coins to another account to register it in the chain
    let tx_cmd = BankCommands::Send {
        to_address: AccAddress::from_bech32("cosmos15jlqmacda2pzerhw48gvvxskweg8sz2saadn99")?,
        amount: Coin::from_str("30uatom")?,
    };
    let Response {
        check_tx: _,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Bank(BankTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(deliver_tx.code.is_ok());

    // create local keypair for second account
    let mnemonic = "utility radio trust maid picture hold palace heart craft fruit recycle void embrace gospel write what soccer resemble yellow decade rug knock control celery";
    let name = "foo";
    key_add(tendermint.1.to_path_buf(), name, mnemonic)?;

    // create destination validator
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"AAAAC3NzaC1lZDI1NTE5AAAAIFFTUWrymqRbtqMGhZACRrr7sWUnqGB8DR+6ob9d0Fhz\"}".to_string();
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    let tx_cmd = StakingCommands::CreateValidator {
        pubkey,
        amount,
        moniker: "foo".to_string(),
        identity: "".to_string(),
        website: "".to_string(),
        security_contact: "".to_string(),
        details: "".to_string(),
        commission_rate: "0.1".to_string(),
        commission_max_rate: "0.2".to_string(),
        commission_max_change_rate: "0.01".to_string(),
        min_self_delegation: Uint256::one(),
    };

    let Response {
        check_tx: _,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: name.to_string(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(deliver_tx.code.is_ok());

    // create delegation to source validator
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    let tx_cmd = StakingCommands::Delegate {
        validator_address: ValAddress::from_bech32(
            "cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk",
        )?,
        amount,
    };
    let Response {
        check_tx: _,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(deliver_tx.code.is_ok());

    /* */

    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    let tx_cmd = StakingCommands::Redelegate {
        src_validator_address: ValAddress::from_bech32(
            "cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk",
        )?,
        dst_validator_address: ValAddress::from_bech32(
            "cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4",
        )?,
        amount,
    };

    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_string(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(check_tx.code.is_ok());
    assert_eq!(check_tx.events.len(), 0);
    assert!(deliver_tx.code.is_ok());
    assert_eq!(deliver_tx.events.len(), 2);
    assert!(deliver_tx.events.iter().any(|e| e.kind == "redelegate"));
    assert_eq!(
        deliver_tx
            .events
            .iter()
            .find(|e| e.kind == "redelegate")
            .unwrap()
            .attributes
            .len(),
        4
    );

    Ok(())
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn redelegate_failed_on_invalid_amount() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;

    // create source validator
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}".to_string();
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(100u64),
    };
    let tx_cmd = StakingCommands::CreateValidator {
        pubkey,
        amount,
        moniker: "test".to_string(),
        identity: "".to_string(),
        website: "".to_string(),
        security_contact: "".to_string(),
        details: "".to_string(),
        commission_rate: "0.1".to_string(),
        commission_max_rate: "0.2".to_string(),
        commission_max_change_rate: "0.01".to_string(),
        min_self_delegation: Uint256::one(),
    };

    let Response {
        check_tx: _,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(deliver_tx.code.is_ok());

    // send coins to another account to register it in the chain
    let tx_cmd = BankCommands::Send {
        to_address: AccAddress::from_bech32("cosmos15jlqmacda2pzerhw48gvvxskweg8sz2saadn99")?,
        amount: Coin::from_str("30uatom")?,
    };
    let Response {
        check_tx: _,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Bank(BankTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(deliver_tx.code.is_ok());

    // create local keypair for second account
    let mnemonic = "utility radio trust maid picture hold palace heart craft fruit recycle void embrace gospel write what soccer resemble yellow decade rug knock control celery";
    let name = "foo";
    key_add(tendermint.1.to_path_buf(), name, mnemonic)?;

    // create destination validator
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"AAAAC3NzaC1lZDI1NTE5AAAAIFFTUWrymqRbtqMGhZACRrr7sWUnqGB8DR+6ob9d0Fhz\"}".to_string();
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    let tx_cmd = StakingCommands::CreateValidator {
        pubkey,
        amount,
        moniker: "foo".to_string(),
        identity: "".to_string(),
        website: "".to_string(),
        security_contact: "".to_string(),
        details: "".to_string(),
        commission_rate: "0.1".to_string(),
        commission_max_rate: "0.2".to_string(),
        commission_max_change_rate: "0.01".to_string(),
        min_self_delegation: Uint256::one(),
    };

    let Response {
        check_tx: _,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: name.to_string(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(deliver_tx.code.is_ok());

    // create delegation to source validator
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    let tx_cmd = StakingCommands::Delegate {
        validator_address: ValAddress::from_bech32(
            "cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk",
        )?,
        amount,
    };
    let Response {
        check_tx: _,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_owned(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(deliver_tx.code.is_ok());

    /* */

    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(11u64),
    };
    let tx_cmd = StakingCommands::Redelegate {
        src_validator_address: ValAddress::from_bech32(
            "cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk",
        )?,
        dst_validator_address: ValAddress::from_bech32(
            "cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4",
        )?,
        amount,
    };

    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: KEY_NAME.to_string(),
                home: tendermint.1.to_path_buf(),
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd })),
        },
        &GaiaCoreClient,
    )?;
    assert!(check_tx.code.is_ok());
    assert!(deliver_tx.code.is_err());
    assert!(deliver_tx.log.contains("invalid shares amount"));
    Ok(())
}
