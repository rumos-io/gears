use std::str::FromStr;

use bank::cli::tx::{BankCommands, BankTxCli};
use gaia_rs::client::GaiaTxCommands;
use gears::{
    commands::client::keys::{keys, AddKeyCommand, KeyCommand},
    extensions::testing::UnwrapTesting,
    tendermint::rpc::response::tx::broadcast::Response,
    types::{
        address::{AccAddress, ValAddress},
        base::coin::UnsignedCoin,
        uint::Uint256,
    },
};
use staking::cli::tx::{StakingCommands, StakingTxCli};
use utilities::GaiaNode;

#[path = "./utilities.rs"]
mod utilities;

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn create_validator() -> anyhow::Result<()> {
    let gaia = GaiaNode::run()?;

    let cmd = helpers::create_validator_tx()?;

    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = gaia
        .tx(cmd, GaiaNode::validator_key())?
        .broadcast()
        .unwrap_test()
        .pop()
        .unwrap_test();

    if deliver_tx.code.is_err() || check_tx.code.is_err() {
        println!("{:#?}", check_tx);
        println!("{:#?}", deliver_tx);
    }

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
    let gaia = GaiaNode::run()?;

    let cmd = helpers::create_validator_tx()?;

    let _ = gaia.tx(cmd, GaiaNode::validator_key())?;

    let amount = UnsignedCoin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };

    // it's the self delegation because function `create_validator_tx` creates a validator with
    // address `cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4` that is a validator address
    // representation of ACC_ADDRESS account address
    let cmd = helpers::new_delegation(
        "cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4",
        amount,
    )?;

    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = gaia
        .tx(cmd, GaiaNode::validator_key())?
        .broadcast()
        .unwrap_test()
        .pop()
        .unwrap_test();

    if deliver_tx.code.is_err() || check_tx.code.is_err() {
        println!("{:#?}", check_tx);
        println!("{:#?}", deliver_tx);
    }

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
    let gaia = GaiaNode::run()?;

    let cmd = helpers::redelegate_tx(&gaia)?;

    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = gaia
        .tx(cmd, GaiaNode::validator_key())?
        .broadcast()
        .unwrap_test()
        .pop()
        .unwrap_test();

    if deliver_tx.code.is_err() || check_tx.code.is_err() {
        println!("{:#?}", check_tx);
        println!("{:#?}", deliver_tx);
    }

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
            .expect("should exists")
            .attributes
            .len(),
        4
    );

    Ok(())
}

#[test]
#[ignore = "rust usually run test in || while this tests be started ony by one"]
fn redelegate_failed_on_invalid_amount() -> anyhow::Result<()> {
    let gaia = GaiaNode::run()?;

    // create source validator
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}";
    let amount = UnsignedCoin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(100u64),
    };
    let cmd = helpers::new_validator(pubkey, amount, "test")?;
    gaia.tx(cmd, GaiaNode::validator_key())?;

    // send coins to another account to register it in the chain
    let tx_cmd = BankCommands::Send {
        to_address: AccAddress::from_bech32("cosmos15jlqmacda2pzerhw48gvvxskweg8sz2saadn99")?,
        amount: UnsignedCoin::from_str("30uatom")?,
    };
    let cmd = GaiaTxCommands::Bank(BankTxCli { command: tx_cmd });
    gaia.tx(cmd, GaiaNode::validator_key())?;

    // create local keypair for second account
    let mnemonic = "utility radio trust maid picture hold palace heart craft fruit recycle void embrace gospel write what soccer resemble yellow decade rug knock control celery";
    let name = "foo";
    keys(KeyCommand::Add(AddKeyCommand {
        name: name.to_owned(),
        recover: true,
        home: gaia.home(),
        keyring_backend: gears::commands::client::keys::KeyringBackend::Test,
        bip39_mnemonic: Some(mnemonic.to_owned()),
    }))?;

    // create destination validator
    let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"AAAAC3NzaC1lZDI1NTE5AAAAIFFTUWrymqRbtqMGhZACRrr7sWUnqGB8DR+6ob9d0Fhz\"}";
    let amount = UnsignedCoin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    let cmd = helpers::new_validator(pubkey, amount, "test")?;
    gaia.tx(cmd, GaiaNode::validator_key())?;

    // create delegation to source validator
    let amount = UnsignedCoin {
        denom: "uatom".try_into()?,
        amount: Uint256::from(10u64),
    };
    let cmd = helpers::new_delegation(
        "cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk",
        amount,
    )?;
    gaia.tx(cmd, GaiaNode::validator_key())?;

    /* test */
    let amount = UnsignedCoin {
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
    let cmd = GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd });
    let Response {
        check_tx,
        deliver_tx,
        hash: _,
        height: _,
    } = gaia
        .tx(cmd, GaiaNode::validator_key())?
        .broadcast()
        .unwrap_test()
        .pop()
        .unwrap_test();

    assert!(check_tx.code.is_ok());
    assert!(deliver_tx.code.is_err());
    assert!(deliver_tx.log.contains("invalid shares amount"));

    Ok(())
}

// fn run_tx_local(
//     from_key: &str,
//     home: PathBuf,
//     command: GaiaTxCommands,
// ) -> anyhow::Result<Response> {
//     // a comment
//     let mut responses = run_tx(
//         TxCommand {
//             inner: WrappedGaiaTxCommands(command),
//             ctx: ClientTxContext::new_online(
//                 home,
//                 200_000_u32.into(),
//                 DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
//                 ChainId::from_str("test-chain")?,
//                 from_key,
//             ),
//         },
//         &GaiaCoreClient,
//         &QueryNodeFetcher,
//     )?
//     .broadcast()
//     .expect("broadcast tx inside");
//     assert_eq!(responses.len(), 1);
//     Ok(responses.pop().expect("vector has exactly single element"))
// }

// fn run_query_local(command: GaiaQueryCommands) -> anyhow::Result<GaiaQueryResponse> {
//     run_query(
//         QueryCommand {
//             node: DEFAULT_TENDERMINT_RPC_ADDRESS.parse()?,
//             height: None,
//             inner: WrappedGaiaQueryCommands(command),
//         },
//         &GaiaCoreClient,
//     )
// }

// fn delegate_tx(home: PathBuf) -> anyhow::Result<Response> {
//     create_validator_tx(home.clone())?;

//     let amount = UnsignedCoin {
//         denom: "uatom".try_into()?,
//         amount: Uint256::from(10u64),
//     };
//     // it's the self delegation because function `create_validator_tx` creates a validator with
//     // address `cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4` that is a validator address
//     // representation of ACC_ADDRESS account address
//     new_delegation(
//         KEY_NAME,
//         home,
//         "cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4",
//         amount,
//     )
// }

// #[test]
// #[ignore = "rust usually run test in || while this tests be started ony by one"]
// fn query_validator() -> anyhow::Result<()> {
//     let (tendermint, _server_thread) =
//         run_gaia_and_tendermint([(acc_address(), default_coin(200_000_000_u32))])?;
//     create_validator_tx(tendermint.1.to_path_buf())?;

//     let query = ValidatorCommand {
//         address: ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")?,
//     };
//     let command = GaiaQueryCommands::Staking(StakingQueryCli {
//         command: QueryStakingCommands::Validator(query),
//     });
//     let Validator {
//         operator_address,
//         delegator_shares,
//         description,
//         consensus_pubkey,
//         jailed,
//         tokens,
//         unbonding_height: _,
//         unbonding_time: _,
//         commission,
//         min_self_delegation,
//         status,
//     } = match run_query_local(command)? {
//         GaiaQueryResponse::Staking(staking::cli::query::StakingQueryResponse::Validator(
//             staking::QueryValidatorResponse { validator: Some(v) },
//         )) => v,
//         _ => unreachable!(),
//     };

//     assert_eq!(
//         operator_address,
//         ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")
//             .expect("hardcoded is valid")
//     );
//     assert_eq!(
//         delegator_shares,
//         Decimal256::from_atomics(100u64, 0).expect("hardcoded is valid")
//     );
//     assert_eq!(
//         description,
//         Description {
//             moniker: "test".into(),
//             ..Default::default()
//         }
//     );
//     assert_eq!(consensus_pubkey, serde_json::from_str("{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}").expect("hardcoded is valid"));
//     assert!(!jailed);
//     assert_eq!(tokens, Uint256::from(100u64));
//     assert_eq!(
//         ibc_proto::cosmos::staking::v1beta1::Commission::from(commission).commission_rates,
//         Some(ibc_proto::cosmos::staking::v1beta1::CommissionRates {
//             rate: 10u64.pow(17).to_string(),
//             max_rate: (2 * 10u64.pow(17)).to_string(),
//             max_change_rate: 10u64.pow(16).to_string(),
//         }),
//     );
//     assert_eq!(min_self_delegation, Uint256::one());
//     assert_eq!(status, BondStatus::Unbonded);
//     Ok(())
// }

// #[test]
// #[ignore = "rust usually run test in || while this tests be started ony by one"]
// fn query_delegation() -> anyhow::Result<()> {
//     let (tendermint, _server_thread) =
//         run_gaia_and_tendermint([(acc_address(), default_coin(200_000_000_u32))])?;

//     // function performs two self delegations:
//     // first is a transaction with creation of a validator: amount 100 uatoms
//     // second is delegation of 10 uatoms to self
//     delegate_tx(tendermint.1.to_path_buf())?;

//     let delegator_address = AccAddress::from_bech32(ACC_ADDRESS)?;
//     let validator_address =
//         ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")?;
//     let query = DelegationCommand {
//         delegator_address: delegator_address.clone(),
//         validator_address: validator_address.clone(),
//     };
//     let command = GaiaQueryCommands::Staking(StakingQueryCli {
//         command: QueryStakingCommands::Delegation(query),
//     });

//     let result = run_query_local(command)?;
//     let expected = GaiaQueryResponse::Staking(
//         staking::cli::query::StakingQueryResponse::Delegation(staking::QueryDelegationResponse {
//             delegation_response: Some(DelegationResponse {
//                 delegation: Some(staking::Delegation {
//                     delegator_address,
//                     validator_address,
//                     shares: Decimal256::from_atomics(110u64, 0).expect("hardcoded is valid"),
//                 }),
//                 balance: Some(UnsignedCoin {
//                     denom: "uatom".try_into().expect("hardcoded is valid"),
//                     amount: Uint256::from(110u64),
//                 }),
//             }),
//         }),
//     );
//     assert_eq!(result, expected);

//     Ok(())
// }

// // TODO: consider to create tests where validator has another bond status
// #[test]
// #[ignore = "rust usually run test in || while this tests be started ony by one"]
// fn query_redelegation() -> anyhow::Result<()> {
//     let (tendermint, _server_thread) =
//         run_gaia_and_tendermint([(acc_address(), default_coin(200_000_000_u32))])?;
//     redelegate_tx(tendermint.1.to_path_buf())?;

//     let delegator_address = AccAddress::from_bech32(ACC_ADDRESS)?;
//     let src_validator_address =
//         ValAddress::from_bech32("cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk")?;
//     let dst_validator_address =
//         ValAddress::from_bech32("cosmosvaloper1syavy2npfyt9tcncdtsdzf7kny9lh777yfrfs4")?;
//     let query = RedelegationCommand {
//         delegator_address: delegator_address.clone(),
//         dst_validator_address: dst_validator_address.clone(),
//         src_validator_address: src_validator_address.clone(),
//     };
//     let command = GaiaQueryCommands::Staking(StakingQueryCli {
//         command: QueryStakingCommands::Redelegation(query),
//     });

//     let result = run_query_local(command)?;
//     // since validator status is BondStatus::Unbonded, the system doesn't store redelegation
//     // entries in the queue
//     let expected =
//         GaiaQueryResponse::Staking(staking::cli::query::StakingQueryResponse::Redelegation(
//             staking::QueryRedelegationResponse {
//                 redelegation_responses: vec![],
//                 pagination: None,
//             },
//         ));
//     assert_eq!(result, expected);

//     Ok(())
// }

mod helpers {
    use std::str::FromStr;

    use bank::cli::tx::{BankCommands, BankTxCli};
    use gaia_rs::client::GaiaTxCommands;
    use gears::{
        commands::client::keys::{keys, AddKeyCommand, KeyCommand},
        types::{
            address::{AccAddress, ValAddress},
            base::coin::UnsignedCoin,
            decimal256::Decimal256,
            uint::Uint256,
        },
    };
    use staking::cli::tx::{CreateValidatorCli, StakingCommands, StakingTxCli};

    use crate::utilities::GaiaNode;

    pub fn create_validator_tx() -> anyhow::Result<GaiaTxCommands> {
        let pubkey = r#"{ "type": "tendermint/PubKeyEd25519", "value": "JVWozgDG2S0TOEE0oFWz/EnSxA0EtYhXQANVIZpePFs="} "#;
        let amount = UnsignedCoin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(100u64),
        };
        // creates a validator, transaction performs self delegation of 100 uatoms
        new_validator(pubkey, amount, "test")
    }

    pub fn new_delegation(
        validator_address: &str,
        amount: UnsignedCoin,
    ) -> anyhow::Result<GaiaTxCommands> {
        let tx_cmd = StakingCommands::Delegate {
            validator_address: ValAddress::from_bech32(validator_address)?,
            amount,
        };
        Ok(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd }))
    }

    pub fn new_validator(
        pubkey: &str,
        amount: UnsignedCoin,
        moniker: &str,
    ) -> anyhow::Result<GaiaTxCommands> {
        let pubkey = serde_json::from_str(pubkey)?;
        let tx_cmd = StakingCommands::CreateValidator(CreateValidatorCli {
            pubkey,
            amount,
            moniker: moniker.to_string(),
            identity: "".to_string(),
            website: "".to_string(),
            security_contact: "".to_string(),
            details: "".to_string(),
            commission_rate: Decimal256::from_atomics(1u64, 1)?,
            commission_max_rate: Decimal256::from_atomics(2u64, 1)?,
            commission_max_change_rate: Decimal256::from_atomics(1u64, 2)?,
            min_self_delegation: Uint256::one(),
        });
        Ok(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd }))
    }

    pub fn redelegate_tx(node: &GaiaNode) -> anyhow::Result<GaiaTxCommands> {
        // create source validator
        let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE=\"}";
        let amount = UnsignedCoin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(100u64),
        };

        node.tx(
            new_validator(pubkey, amount, "test")?,
            GaiaNode::validator_key(),
        )?;

        // send coins to another account to register it in the chain
        let tx_cmd = BankCommands::Send {
            to_address: AccAddress::from_bech32("cosmos15jlqmacda2pzerhw48gvvxskweg8sz2saadn99")?,
            amount: UnsignedCoin::from_str("30uatom")?,
        };
        node.tx(
            GaiaTxCommands::Bank(BankTxCli { command: tx_cmd }),
            GaiaNode::validator_key(),
        )?;

        // create local keypair for second account
        let mnemonic = "utility radio trust maid picture hold palace heart craft fruit recycle void embrace gospel write what soccer resemble yellow decade rug knock control celery";
        let name = "foo";
        keys(KeyCommand::Add(AddKeyCommand {
            name: name.to_owned(),
            recover: true,
            home: node.home(),
            keyring_backend: gears::commands::client::keys::KeyringBackend::Test,
            bip39_mnemonic: Some(mnemonic.to_owned()),
        }))?;

        // create destination validator
        let pubkey = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"AAAAC3NzaC1lZDI1NTE5AAAAIFFTUWrymqRbtqMGhZACRrr7sWUnqGB8DR+6ob9d0Fhz\"}";
        let amount = UnsignedCoin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(10u64),
        };
        let cmd = new_validator(pubkey, amount, "test")?;
        let _ = node.tx(cmd, GaiaNode::validator_key())?;

        // create delegation to source validator
        let amount = UnsignedCoin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(10u64),
        };
        let cmd = new_delegation(
            "cosmosvaloper15jlqmacda2pzerhw48gvvxskweg8sz2scfexfk",
            amount,
        )?;
        let _ = node.tx(cmd, GaiaNode::validator_key())?;

        /* test */
        let amount = UnsignedCoin {
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

        Ok(GaiaTxCommands::Staking(StakingTxCli { command: tx_cmd }))
    }
}
