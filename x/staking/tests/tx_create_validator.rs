use std::str::FromStr;

use gears::{
    core::Protobuf,
    extensions::testing::UnwrapTesting,
    tendermint::types::{
        request::query::RequestQuery, response::ResponseDeliverTx, time::timestamp::Timestamp,
    },
    types::{base::coin::UnsignedCoin, decimal256::Decimal256, uint::Uint256},
    utils::node::{generate_tx, GenesisSource, StepResponse, User},
    x::types::validator::BondStatus,
};

use staking::{
    Commission, CommissionRates, CreateValidator, Description, IbcV046Validator, Message,
    QueryValidatorRequest, QueryValidatorResponse,
};
use utils::{set_node, CONSENSUS_KEY, CONSENSUS_PUBLIC_KEY};

#[path = "./utils.rs"]
mod utils;

const GENESIS_FILE_PATH: &str = "./tests/assets/tx_create_validator.json";

#[test]
fn create_validator_unbounded() {
    let mut node = set_node(GenesisSource::File(GENESIS_FILE_PATH.into()));

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let user = User::from_bech32("unfair live spike near cushion blanket club salad poet cigar venue above north speak harbor salute curve tail appear obvious month end boss priority", 1).unwrap_test();

    let commission = CommissionRates::new(
        Decimal256::from_atomics(1u64, 1).unwrap_test(),
        Decimal256::from_atomics(2u64, 1).unwrap_test(),
        Decimal256::from_atomics(1u64, 2).unwrap_test(),
    )
    .unwrap_test();

    let msg = CreateValidator {
        description: Description::try_new("test", "", "", "", "").unwrap_test(),
        commission: commission.clone(),
        min_self_delegation: Uint256::one(),
        delegator_address: user.address(),
        validator_address: user.address().into(),
        pubkey: serde_json::from_str(CONSENSUS_KEY).unwrap_test(),
        value: UnsignedCoin::from_str("100uatom").unwrap_test(),
    };

    let txs = generate_tx(
        vec1::vec1![Message::CreateValidator(msg)],
        0,
        &user,
        node.chain_id().clone(),
    );

    let StepResponse {
        app_hash,
        mut tx_responses,
        height: _,
    } = node.step(vec![txs], Timestamp::UNIX_EPOCH);

    let ResponseDeliverTx { code, log, .. } = tx_responses.pop().unwrap_test();

    assert!(code == 0, "{log}");

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "615fc7b99bc5375281010fcb424e0dbc4b9145b2e386396f5fc9c414b5d734b3"
    );

    let StepResponse {
        app_hash,
        tx_responses: _,
        height: _,
    } = node.step(vec![], Timestamp::UNIX_EPOCH);

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "22b2a66570e221f9c79209432c46d8a14c730c8ff4d5a5e20b67e73e50e504fe"
    );

    let q = QueryValidatorRequest {
        validator_addr: user.address().into(),
    };
    let query_result = node.query(RequestQuery {
        data: q.encode_vec().into(),
        path: QueryValidatorRequest::QUERY_URL.to_owned(),
        height: node.height() as i64,
        prove: false,
    });

    let validator = match QueryValidatorResponse::decode(query_result.value)
        .unwrap_test()
        .validator
    {
        Some(deposit) => deposit,
        None => panic!("failed to find validator"),
    };

    let expected_validator = IbcV046Validator {
        operator_address: user.address().into(),
        delegator_shares: Decimal256::from_atomics(100_u32, 0).unwrap_test(),
        description: Description::try_new("test", "", "", "", "").unwrap_test(),
        consensus_pubkey: serde_json::from_str(CONSENSUS_PUBLIC_KEY).unwrap_test(),
        jailed: false,
        tokens: Uint256::from(100_u32),
        unbonding_height: 0,
        unbonding_time: Timestamp::UNIX_EPOCH,
        commission: Commission::new(commission, Timestamp::UNIX_EPOCH),
        min_self_delegation: Uint256::one(),
        status: BondStatus::Unbonded,
        unbonding_ids: Vec::new(),
        unbonding_on_hold_ref_count: Uint256::zero(),
        validator_bond_shares: Decimal256::zero(),
        liquid_shares: Decimal256::zero(),
    };

    pretty_assertions::assert_eq!(expected_validator, validator);
}
