use std::str::FromStr;

use gears::{
    core::Protobuf,
    extensions::testing::UnwrapTesting,
    tendermint::types::{
        request::query::RequestQuery, response::ResponseDeliverTx, time::timestamp::Timestamp,
    },
    types::{base::coin::UnsignedCoin, decimal256::Decimal256, uint::Uint256},
    utils::node::{generate_tx, StepResponse, User},
    x::types::validator::BondStatus,
};

use staking::{
    Commission, CommissionRates, CreateValidator, Description, IbcV046Validator, Message,
    QueryValidatorRequest, QueryValidatorResponse,
};
use utils::set_node;

#[path = "./utils.rs"]
mod utils;

const CONSENSUS_KEY: &str = r#"{ "type": "tendermint/PubKeyEd25519", "value": "JVWozgDG2S0TOEE0oFWz/EnSxA0EtYhXQANVIZpePFs="} "#;
const CONSENSUS_PUBLIC_KEY : &str = "{\"@type\":\"/cosmos.crypto.ed25519.PubKey\",\"key\":\"JVWozgDG2S0TOEE0oFWz/EnSxA0EtYhXQANVIZpePFs=\"}";

#[test]
fn create_validator_unbounded() {
    let mut node = set_node();

    let _ = node.step(vec![], Timestamp::UNIX_EPOCH);

    let user = User::from_bech32("embrace dash dirt awake weird beauty nest fee slice reopen hundred width bright glass kick also forget forum guess guard unusual poet grass very", 1).unwrap_test();

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
    } = node.step(vec![txs], Timestamp::UNIX_EPOCH);

    let ResponseDeliverTx { code, .. } = tx_responses.pop().unwrap_test();

    assert!(code == 0);

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "83f9c3a55de321b0d7970ccdaa7719c95af28fa0f6de8d6928e649c128749914"
    );

    let StepResponse {
        app_hash,
        tx_responses: _,
    } = node.step(vec![], Timestamp::UNIX_EPOCH);

    assert_eq!(
        data_encoding::HEXLOWER.encode(&app_hash),
        "00327c5505998e3ce4640656a09bfce8f099947a9717b91ae53c9d91da60c0cc"
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
