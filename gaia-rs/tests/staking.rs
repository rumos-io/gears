use crate::utilities::{key_add, run_gaia_and_tendermint, KEY_NAME};
use bank::cli::tx::{BankCommands, BankTxCli};
use gaia_rs::{
    client::{GaiaQueryCommands, GaiaTxCommands, WrappedGaiaQueryCommands, WrappedGaiaTxCommands},
    query::GaiaQueryResponse,
    GaiaCoreClient,
};
use gears::{
    commands::client::{
        keys::KeyringBackend,
        query::{run_query, QueryCommand},
        tx::{run_tx, Keyring, LocalInfo, TxCommand},
    },
    config::DEFAULT_TENDERMINT_RPC_ADDRESS,
    tendermint::{rpc::response::tx::broadcast::Response, types::chain_id::ChainId},
    types::{
        address::{AccAddress, ValAddress},
        base::coin::Coin,
        decimal256::Decimal256,
        uint::Uint256,
    },
    x::types::validator::BondStatus,
};
use staking::{
    cli::{
        query::{
            DelegationCommand, RedelegationCommand, StakingCommands as QueryStakingCommands,
            StakingQueryCli, ValidatorCommand,
        },
        tx::{StakingCommands, StakingTxCli},
    },
    CommissionRatesRaw, CommissionRaw, DelegationResponse, Description, Validator,
};
use std::{path::PathBuf, str::FromStr};
use utilities::ACC_ADDRESS;

#[path = "./utilities.rs"]
mod utilities;

fn run_tx_local(
    from_key: &str,
    home: PathBuf,
    command: GaiaTxCommands,
) -> anyhow::Result<Response> {
    run_tx(
        TxCommand {
            keyring: Keyring::Local(LocalInfo {
                keyring_backend: KeyringBackend::Test,
                from_key: from_key.to_owned(),
                home,
            }),
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            chain_id: ChainId::from_str("test-chain")?,
            fee: None,
            inner: WrappedGaiaTxCommands(command),
        },
        &GaiaCoreClient,
    )
}

fn run_query_local(command: GaiaQueryCommands) -> anyhow::Result<GaiaQueryResponse> {
    run_query(
        QueryCommand {
            node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
            height: None,
            inner: WrappedGaiaQueryCommands(command),
        },
        &GaiaCoreClient,
    )
}

fn new_validator(
    from_key: &str,
    home: PathBuf,
    pubkey: &str,
    amount: Coin,
    moniker: &str,
) -> anyhow::Result<Response> {
    let tx_cmd = StakingCommands::CreateValidator {
        pubkey: pubkey.to_string(),
        amount,
        moniker: moniker.to_string(),
        identity: "".to_string(),
        website: "".to_string(),
        security_contact: "".to_string(),
        details: "".to_string(),
        commission_rate: "0.1".to_string(),
        commission_max_rate: "0.2".to_string(),
        commission_max_change_rate: "0.01".to_string(),
        min_self_delegation: Uint256::one(),
    };
    let command = GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd });
    run_tx_local(from_key, home, command)
}

fn new_delegation(
    from_key: &str,
    home: PathBuf,
    validator_address: &str,
    amount: Coin,
) -> anyhow::Result<Response> {
    let tx_cmd = StakingCommands::Delegate {
        validator_address: ValAddress::from_bech32(validator_address)?,
        amount,
    };
    let command = GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd });
    run_tx_local(from_key, home, command)
}

fn create_validator_tx(home: PathBuf) -> anyhow::Result<Response> {
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}";
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(100u64),
    };
    new_validator(KEY_NAME, home, pubkey, amount, "test")
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn create_validator() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;
    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = create_validator_tx(tendermint.1.to_path_buf())?;
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

fn delegate_tx(home: PathBuf) -> anyhow::Result<Response> {
    create_validator_tx(home.clone())?;

    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    new_delegation(
        KEY_NAME,
        home,
        "cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4",
        amount,
    )
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn delegate() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;

    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = delegate_tx(tendermint.1.to_path_buf())?;

    assert!(check_tx.code.is_ok());
    assert_eq!(check_tx.events.len(), 0);
    assert!(deliver_tx.code.is_ok());
    assert_eq!(deliver_tx.events.len(), 4);
    assert!(deliver_tx.events.iter().any(|e| e.kind == "delegate"));
    assert!(deliver_tx.events.iter().any(|e| e.kind == "coin_spent"));
    assert!(deliver_tx.events.iter().any(|e| e.kind == "coin_received"));

    Ok(())
}

fn redelegate_tx(home: PathBuf) -> anyhow::Result<Response> {
    // create source validator
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}";
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(100u64),
    };
    new_validator(KEY_NAME, home.clone(), pubkey, amount, "test")?;

    // send coins to another account to register it in the chain
    let tx_cmd = BankCommands::Send {
        to_address: AccAddress::from_bech32("cosmos15jlqmacda2pzerhw48gvvxskweg8sz2saadn99")?,
        amount: Coin::from_str("30uatom")?,
    };
    let command = GaiaTxCommands::Bank(BankTxCli { command: tx_cmd });
    run_tx_local(KEY_NAME, home.clone(), command)?;

    // create local keypair for second account
    let mnemonic = "utility radio trust maid picture hold palace heart craft fruit recycle void embrace gospel write what soccer resemble yellow decade rug knock control celery";
    let name = "foo";
    key_add(home.clone(), name, mnemonic)?;

    // create destination validator
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"AAAAC3NzaC1lZDI1NTE5AAAAIFFTUWrymqRbtqMGhZACRrr7sWUnqGB8DR+6ob9d0Fhz\"}";
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    new_validator(name, home.clone(), pubkey, amount, name)?;

    // create delegation to source validator
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    new_delegation(
        KEY_NAME,
        home.clone(),
        "cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk",
        amount,
    )?;

    /* test */

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

    let command = GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd });
    run_tx_local(KEY_NAME, home, command)
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn redelegate() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;

    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = redelegate_tx(tendermint.1.to_path_buf())?;

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
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}";
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(100u64),
    };
    new_validator(KEY_NAME, tendermint.1.to_path_buf(), pubkey, amount, "test")?;

    // send coins to another account to register it in the chain
    let tx_cmd = BankCommands::Send {
        to_address: AccAddress::from_bech32("cosmos15jlqmacda2pzerhw48gvvxskweg8sz2saadn99")?,
        amount: Coin::from_str("30uatom")?,
    };
    let command = GaiaTxCommands::Bank(BankTxCli { command: tx_cmd });
    run_tx_local(KEY_NAME, tendermint.1.to_path_buf(), command)?;

    // create local keypair for second account
    let mnemonic = "utility radio trust maid picture hold palace heart craft fruit recycle void embrace gospel write what soccer resemble yellow decade rug knock control celery";
    let name = "foo";
    key_add(tendermint.1.to_path_buf(), name, mnemonic)?;

    // create destination validator
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"AAAAC3NzaC1lZDI1NTE5AAAAIFFTUWrymqRbtqMGhZACRrr7sWUnqGB8DR+6ob9d0Fhz\"}";
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    new_validator(KEY_NAME, tendermint.1.to_path_buf(), pubkey, amount, name)?;

    // create delegation to source validator
    let amount = Coin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    new_delegation(
        KEY_NAME,
        tendermint.1.to_path_buf(),
        "cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk",
        amount,
    )?;

    /* test */

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
    let command = GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd });
    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = run_tx_local(KEY_NAME, tendermint.1.to_path_buf(), command)?;

    assert!(check_tx.code.is_ok());
    assert!(deliver_tx.code.is_err());
    assert!(deliver_tx.log.contains("invalid shares amount"));
    Ok(())
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn query_validator() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;
    create_validator_tx(tendermint.1.to_path_buf())?;

    let query = ValidatorCommand {
        address: ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")?,
    };
    let command = GaiaQueryCommands::Staking(StakingQueryCli {
        command: QueryStakingCommands::Validator(query),
    });
    let Validator {
        operator_address,
        delegator_shares,
        description,
        consensus_pubkey,
        jailed,
        tokens,
        unbonding_height: _,
        unbonding_time: _,
        commission,
        min_self_delegation,
        status,
    } = match run_query_local(command)? {
        GaiaQueryResponse::Staking(staking::cli::query::StakingQueryResponse::Validator(
            staking::QueryValidatorResponse { validator: Some(v) },
        )) => v,
        _ => unreachable!(),
    };

    assert_eq!(
        operator_address,
        ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4").unwrap()
    );
    assert_eq!(
        delegator_shares,
        Decimal256::from_atomics(100u64, 0).unwrap()
    );
    assert_eq!(
        description,
        Description {
            moniker: "test".into(),
            ..Default::default()
        }
    );
    assert_eq!(consensus_pubkey, serde_json::from_str("{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}").unwrap());
    assert_eq!(jailed, false);
    assert_eq!(tokens, Uint256::from(100u64));
    assert_eq!(
        CommissionRaw::from(commission).commission_rates,
        Some(CommissionRatesRaw {
            rate: 10u64.pow(17).to_string(),
            max_rate: (2 * 10u64.pow(17)).to_string(),
            max_change_rate: (1 * 10u64.pow(16)).to_string(),
        }),
    );
    assert_eq!(min_self_delegation, Uint256::one());
    assert_eq!(status, BondStatus::Unbonded);
    Ok(())
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn query_delegation() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;
    delegate_tx(tendermint.1.to_path_buf())?;

    let delegator_address = AccAddress::from_bech32(ACC_ADDRESS)?;
    let validator_address =
        ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")?;
    let query = DelegationCommand {
        delegator_address: delegator_address.clone(),
        validator_address: validator_address.clone(),
    };
    let command = GaiaQueryCommands::Staking(StakingQueryCli {
        command: QueryStakingCommands::Delegation(query),
    });

    let result = run_query_local(command)?;
    let expected = GaiaQueryResponse::Staking(
        staking::cli::query::StakingQueryResponse::Delegation(staking::QueryDelegationResponse {
            delegation_response: Some(DelegationResponse {
                delegation: staking::Delegation {
                    delegator_address,
                    validator_address,
                    shares: Decimal256::from_atomics(10u64, 0).unwrap(),
                },
                balance: Coin {
                    denom: "uatom".try_into().unwrap(),
                    amount: Uint256::from(10u64),
                },
            }),
        }),
    );
    assert_eq!(result, expected);

    Ok(())
}

// TODO: consider to create tests where validator has another bond status
#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn query_redelegation() -> anyhow::Result<()> {
    let coins = 200_000_000_u32;
    let (tendermint, _server_thread) = run_gaia_and_tendermint(coins)?;
    redelegate_tx(tendermint.1.to_path_buf())?;

    let delegator_address = AccAddress::from_bech32(ACC_ADDRESS)?;
    let src_validator_address =
        ValAddress::from_bech32("cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk")?;
    let dst_validator_address =
        ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")?;
    let query = RedelegationCommand {
        delegator_address: delegator_address.clone(),
        dst_validator_address: dst_validator_address.clone(),
        src_validator_address: src_validator_address.clone(),
    };
    let command = GaiaQueryCommands::Staking(StakingQueryCli {
        command: QueryStakingCommands::Redelegation(query),
    });

    let result = run_query_local(command)?;
    // since validator status is BondStatus::Unbonded, the system doesn't store redelegation
    // entries in the queue
    let expected =
        GaiaQueryResponse::Staking(staking::cli::query::StakingQueryResponse::Redelegation(
            staking::QueryRedelegationResponse {
                redelegation_responses: vec![],
                pagination: None,
            },
        ));
    assert_eq!(result, expected);

    Ok(())
}
