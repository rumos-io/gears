use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// gears::core::base::coin::Coin has wrong order of fields
// It is better to create a struct with correct order than
// reorder fields by some dynamic struct
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Coin {
    pub amount: String,
    pub denom: String,
}

/// StdFee includes the amount of coins paid in fees and the maximum
/// gas to be used by the transaction. The ratio yields an effective "gasprice",
/// which must be above some miminum to be accepted into the mempool.
/// [Deprecated]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StdFee {
    pub amount: Vec<Coin>,
    pub gas: String,
    // field for compatibility with keplr std fee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granter: Option<String>,
    // field for compatibility with keplr std fee
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "feePayer")]
    pub payer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Msg {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: Map<String, Value>,
}

/// StdSignDoc is replay-prevention structure.
/// It includes the result of msg.get_sign_bytes(),
/// as well as the ChainID (prevent cross chain replay)
/// and the Sequence numbers for each signature (prevent
/// inchain replay and enforce tx ordering per account).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StdSignDoc {
    pub account_number: String,
    pub chain_id: String,
    pub fee: StdFee,
    pub memo: String,
    pub msgs: Vec<Msg>,
    pub sequence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_height: Option<String>,
}

pub fn proto_type_url_to_legacy_amino_type_url(kind: &str) -> String {
    // it seems like the other transactions will have it's amino type
    match kind {
        "/cosmos.bank.v1beta1.MsgSend" => "cosmos-sdk/MsgSend".to_string(),
        _ => kind.to_string(),
    }
}

#[cfg(test)]
mod test {
    use crate::crypto::secp256k1::Secp256k1PubKey;

    use super::*;

    #[test]
    fn parse_verify_send_transaction() -> anyhow::Result<()> {
        let any_json_std_sign_doc_str = "{\"chain_id\":\"test-chain\",\"account_number\":\"5\",\"sequence\":\"0\",\"fee\":{\"gas\":\"200000\",\"amount\":[{\"amount\":\"2000\",\"denom\":\"uatom\"}]},\"msgs\":[{\"type\":\"cosmos-sdk/MsgSend\",\"value\":{\"from_address\":\"cosmos1rm96mrd64yykxyuprjlcxa4yr4llph0rpg27vy\",\"to_address\":\"cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux\",\"amount\":[{\"amount\":\"1\",\"denom\":\"uatom\"}]}}],\"memo\":\"\"}";
        let std_sign_doc: StdSignDoc = serde_json::from_str(any_json_std_sign_doc_str)?;

        let pk: Secp256k1PubKey = serde_json::from_str(
            r#"{
        "key":"AtTjh9XR+GbfnqBXHb1Gcj2a6i3oWlTiqT0SDIDWGYFR"
        }"#,
        )
        .unwrap();

        let bytes = serde_json::to_vec(&std_sign_doc).unwrap();
        let exp = vec![
            123, 34, 97, 99, 99, 111, 117, 110, 116, 95, 110, 117, 109, 98, 101, 114, 34, 58, 34,
            53, 34, 44, 34, 99, 104, 97, 105, 110, 95, 105, 100, 34, 58, 34, 116, 101, 115, 116,
            45, 99, 104, 97, 105, 110, 34, 44, 34, 102, 101, 101, 34, 58, 123, 34, 97, 109, 111,
            117, 110, 116, 34, 58, 91, 123, 34, 97, 109, 111, 117, 110, 116, 34, 58, 34, 50, 48,
            48, 48, 34, 44, 34, 100, 101, 110, 111, 109, 34, 58, 34, 117, 97, 116, 111, 109, 34,
            125, 93, 44, 34, 103, 97, 115, 34, 58, 34, 50, 48, 48, 48, 48, 48, 34, 125, 44, 34,
            109, 101, 109, 111, 34, 58, 34, 34, 44, 34, 109, 115, 103, 115, 34, 58, 91, 123, 34,
            116, 121, 112, 101, 34, 58, 34, 99, 111, 115, 109, 111, 115, 45, 115, 100, 107, 47, 77,
            115, 103, 83, 101, 110, 100, 34, 44, 34, 118, 97, 108, 117, 101, 34, 58, 123, 34, 97,
            109, 111, 117, 110, 116, 34, 58, 91, 123, 34, 97, 109, 111, 117, 110, 116, 34, 58, 34,
            49, 34, 44, 34, 100, 101, 110, 111, 109, 34, 58, 34, 117, 97, 116, 111, 109, 34, 125,
            93, 44, 34, 102, 114, 111, 109, 95, 97, 100, 100, 114, 101, 115, 115, 34, 58, 34, 99,
            111, 115, 109, 111, 115, 49, 114, 109, 57, 54, 109, 114, 100, 54, 52, 121, 121, 107,
            120, 121, 117, 112, 114, 106, 108, 99, 120, 97, 52, 121, 114, 52, 108, 108, 112, 104,
            48, 114, 112, 103, 50, 55, 118, 121, 34, 44, 34, 116, 111, 95, 97, 100, 100, 114, 101,
            115, 115, 34, 58, 34, 99, 111, 115, 109, 111, 115, 49, 115, 121, 97, 118, 121, 50, 110,
            112, 102, 121, 116, 57, 116, 99, 110, 99, 100, 116, 115, 100, 122, 102, 55, 107, 110,
            121, 57, 108, 104, 55, 55, 55, 112, 97, 104, 117, 117, 120, 34, 125, 125, 93, 44, 34,
            115, 101, 113, 117, 101, 110, 99, 101, 34, 58, 34, 48, 34, 125,
        ];
        assert_eq!(bytes, exp);

        let signature = vec![
            144, 45, 103, 232, 109, 21, 6, 198, 235, 73, 138, 23, 6, 89, 70, 239, 214, 1, 220, 132,
            146, 165, 193, 22, 102, 208, 243, 128, 182, 207, 93, 54, 31, 107, 228, 88, 193, 45,
            165, 63, 240, 112, 65, 43, 137, 144, 135, 223, 98, 206, 226, 166, 187, 4, 14, 229, 195,
            69, 152, 212, 59, 137, 37, 12,
        ];
        pk.verify_signature(bytes, signature)?;

        Ok(())
    }
}
