use std::collections::BTreeMap;
use std::fmt::Display;

use crate::signing::renderer::tx::Envelope;
use crate::types::denom::Denom;
use crate::types::{
    rendering::screen::Screen,
    tx::{data::TxData, metadata::Metadata, signer::SignerData, TxMessage},
};
use ciborium::{value::CanonicalValue, Value};

use super::{errors::SigningErrors, renderer::value_renderer::ValueRenderer};

pub trait MetadataGetter {
    type Error: Display;

    fn metadata(&self, denom: &Denom) -> Result<Option<Metadata>, Self::Error>;
}

#[derive(Debug)]
pub struct SignModeHandler;

impl SignModeHandler {
    pub fn sign_bytes_get<MG: MetadataGetter>(
        &self,
        get_metadata: &MG,
        signer_data: SignerData,
        tx_data: TxData<impl TxMessage + ValueRenderer>,
    ) -> Result<Vec<u8>, SigningErrors> {
        let data = Envelope::new(signer_data, tx_data);

        let screens = data
            .format(get_metadata)
            .map_err(|e| SigningErrors::CustomError(e.to_string()))?;

        let map = screens.iter().map(Screen::cbor_map).collect::<Vec<_>>();

        let canonical_key: CanonicalValue = Value::Integer(1.into()).into();
        let mut final_map = BTreeMap::new();
        final_map.insert(canonical_key, map);

        let mut bytes = Vec::new();

        ciborium::into_writer(&final_map, &mut bytes)
            .map_err(|e| SigningErrors::CustomError(e.to_string()))?;

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::signing::handler::SignModeHandler;
    use crate::signing::renderer::test_functions::{TestMetadataGetter, TestNoneMetadataGetter};
    use crate::types::address::AccAddress;
    use crate::types::denom::Denom;
    use crate::types::{
        auth::{fee::Fee, info::AuthInfo},
        base::{coin::UnsignedCoin, coins::UnsignedCoins},
        msg::send::MsgSend,
        rendering::{
            cbor::Cbor,
            screen::{Content, Indent, Screen},
        },
        signing::SignerInfo,
        tx::{body::TxBody, data::TxData, signer::SignerData},
    };
    use ciborium::Value;
    use core_types::tx::mode_info::{ModeInfo, SignMode};
    use cosmwasm_std::Uint256;
    use std::{collections::BTreeMap, str::FromStr};
    use tendermint::types::chain_id::ChainId;
    use vec1::vec1;

    #[test]
    fn test_sign_bytes_with_fmt() -> anyhow::Result<()> {
        let signer_info = SignerInfo {
            public_key: Some(serde_json::from_str(
                r#"{
                        "@type": "/cosmos.crypto.secp256k1.PubKey",
                        "key": "Auvdf+T963bciiBe9l15DNMOijdaXCUo6zqSOvH7TXlN"
                    }"#,
            )?),
            mode_info: ModeInfo::Single(SignMode::Textual),
            sequence: 2,
        };

        let auth_inf = AuthInfo {
            signer_infos: vec![signer_info],
            fee: Fee {
                amount: Some(
                    UnsignedCoins::new(vec![UnsignedCoin {
                        denom: Denom::try_from("uatom".to_owned())?,
                        amount: Uint256::from(2000u32),
                    }])
                    .unwrap(),
                ),
                gas_limit: 100000_u64.try_into().expect("this is a valid gas limit"),
                payer: None,
                granter: String::new(),
            },
            tip: None,
        };

        let signer_data = SignerData {
            address: AccAddress::from_bech32("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
            chain_id: ChainId::from_str("my-chain").expect("this is a valid chain id"),
            account_number: 1,
            sequence: 2,
            pub_key: serde_json::from_str(
                r#"{
                    "@type": "/cosmos.crypto.secp256k1.PubKey",
				"key": "Auvdf+T963bciiBe9l15DNMOijdaXCUo6zqSOvH7TXlN"
			}"#,
            )?,
        };

        let tx_body = TxBody::<MsgSend> {
            messages: vec1![MsgSend {
                from_address: AccAddress::from_bech32(
                    "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
                )?,
                to_address: AccAddress::from_bech32(
                    "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t",
                )?,
                amount: UnsignedCoins::new(vec![UnsignedCoin {
                    denom: Denom::try_from("uatom".to_string())?,
                    amount: Uint256::from(10000000u32),
                }])
                .unwrap(),
            }],
            memo: String::new(),
            timeout_height: 0,
            extension_options: Vec::new(),
            non_critical_extension_options: Vec::new(),
        };

        let tx_data = TxData::<MsgSend> {
            body: tx_body,
            auth_info: auth_inf,
        };

        let handler = SignModeHandler;

        let cbor = handler.sign_bytes_get(&TestMetadataGetter, signer_data, tx_data)?;

        const EXPECTED_CBOR : &str = "a1018fa20168436861696e20696402686d792d636861696ea2016e4163636f756e74206e756d626572026131a2016853657175656e6365026132a301674164647265737302782d636f736d6f7331756c6176336873656e7570737771666b77327933737570356b677471776e767161386579687304f5a3016a5075626c6963206b657902781f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657904f5a401634b657902785230324542204444374620453446442045423736204443384120323035452046363544203739304320443330452038413337203541354320323532382045423341203932334120463146422034443739203444030104f5a102781e54686973207472616e73616374696f6e206861732031204d657373616765a3016d4d6573736167652028312f312902781c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e640301a3016c46726f6d206164647265737302782d636f736d6f7331756c6176336873656e7570737771666b77327933737570356b677471776e76716138657968730302a3016a546f206164647265737302782d636f736d6f7331656a726634637572327779366b667572673966326a707070326833616665356836706b6835740302a30166416d6f756e74026731302041544f4d0302a1026e456e64206f66204d657373616765a2016446656573026a302e3030322041544f4da30169476173206c696d697402673130302730303004f5a3017148617368206f66207261772062797465730278403738356264333036656138393632636462393630303038396264643635663364633032396531616561313132646565363965313935343663396164616438366504f5";

        validate_result([(cbor, EXPECTED_CBOR)]);

        Ok(())
    }

    #[test]
    fn test_sign_bytes_with_screens() -> anyhow::Result<()> {
        const EXPECTED_CBOR : &str = "a1018fa20168436861696e20696402686d792d636861696ea2016e4163636f756e74206e756d626572026131a2016853657175656e6365026132a301674164647265737302782d636f736d6f7331756c6176336873656e7570737771666b77327933737570356b677471776e767161386579687304f5a3016a5075626c6963206b657902781f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657904f5a401634b657902785230324542204444374620453446442045423736204443384120323035452046363544203739304320443330452038413337203541354320323532382045423341203932334120463146422034443739203444030104f5a102781e54686973207472616e73616374696f6e206861732031204d657373616765a3016d4d6573736167652028312f312902781c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e640301a3016c46726f6d206164647265737302782d636f736d6f7331756c6176336873656e7570737771666b77327933737570356b677471776e76716138657968730302a3016a546f206164647265737302782d636f736d6f7331656a726634637572327779366b667572673966326a707070326833616665356836706b6835740302a30166416d6f756e74026731302041544f4d0302a1026e456e64206f66204d657373616765a2016446656573026a302e3030322041544f4da30169476173206c696d697402673130302730303004f5a3017148617368206f66207261772062797465730278403738356264333036656138393632636462393630303038396264643635663364633032396531616561313132646565363965313935343663396164616438366504f5";

        let screens = vec![
            Screen {
                title: "Chain id".to_string(),
                content: Content::try_new("my-chain".to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Account number".to_string(),
                content: Content::try_new(1.to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Sequence".to_string(),
                content: Content::try_new(2.to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Address".to_string(),
                content: Content::try_new("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Public key".to_string(),
                content: Content::try_new("/cosmos.crypto.secp256k1.PubKey")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Key".to_string(),
                content: Content::try_new( "02EB DD7F E4FD EB76 DC8A 205E F65D 790C D30E 8A37 5A5C 2528 EB3A 923A F1FB 4D79 4D" )?,
                indent: Some(Indent::try_new(1)?),
                expert: true,
            },
            Screen {
                title: String::new(),
                content: Content::try_new("This transaction has 1 Message")?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Message (1/1)".to_string(),
                content: Content::try_new("/cosmos.bank.v1beta1.MsgSend")?,
                indent: Some(Indent::try_new(1)?),
                expert: false,
            },
            Screen {
                title: "From address".to_string(),
                content: Content::try_new("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
                indent: Some(Indent::try_new(2)?),
                expert: false,
            },
            Screen {
                title: "To address".to_string(),
                content: Content::try_new("cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t")?,
                indent: Some(Indent::try_new(2)?),
                expert: false,
            },
            Screen {
                title: "Amount".to_string(),
                content: Content::try_new("10 ATOM")?,
                indent: Some(Indent::try_new(2)?),
                expert: false,
            },
            Screen {
                title: String::new(),
                content: Content::try_new("End of Message")?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Fees".to_string(),
                content: Content::try_new("0.002 ATOM".to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Gas limit".to_string(),
                content: Content::try_new("100'000".to_string())?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Hash of raw bytes".to_string(),
                content: Content::try_new(
                    "785bd306ea8962cdb9600089bdd65f3dc029e1aea112dee69e19546c9adad86e",
                )?,
                indent: None,
                expert: true,
            },
        ];

        let map = screens.iter().map(Screen::cbor_map).collect::<Vec<_>>();

        let mut final_map = BTreeMap::new();

        final_map.insert(Value::Integer(1.into()).into(), map);
        let mut bytes = Vec::new();

        final_map.encode(&mut bytes)?;

        validate_result([(bytes, EXPECTED_CBOR)]);

        Ok(())
    }

    fn validate_result<'a>(value: impl IntoIterator<Item = (Vec<u8>, &'a str)>) {
        for (i, expected) in value {
            let actual = data_encoding::HEXLOWER.encode(&i);
            assert_eq!(actual, expected.to_string(), "actual != expected");
        }
    }

    #[test]
    fn test_sign_bytes_works() -> anyhow::Result<()> {
        let signer_info = SignerInfo {
            public_key: Some(serde_json::from_str(
                r#"{
                        "@type": "/cosmos.crypto.secp256k1.PubKey",
                        "key": "A7Jg0Wg+RHwI7CAkSbCjpfWFROGtYYkUlaBVxCT6UXJ4"
                    }"#,
            )?),
            mode_info: ModeInfo::Single(SignMode::Textual),
            sequence: 6,
        };

        let auth_inf = AuthInfo {
            signer_infos: vec![signer_info],
            fee: Fee {
                amount: None,
                gas_limit: 200000_u64.try_into().expect("this is a valid gas limit"),
                payer: None,
                granter: String::new(),
            },
            tip: None,
        };

        let signer_data = SignerData {
            address: AccAddress::from_bech32("cosmos12vrgunwvszgzpykdrqlx3m6puedvcajlxcyw8z")?,
            chain_id: ChainId::from_str("test-chain").expect("this is a valid chain id"),
            account_number: 8,
            sequence: 6,
            pub_key: serde_json::from_str(
                r#"{
                    "@type": "/cosmos.crypto.secp256k1.PubKey",
        		"key": "A7Jg0Wg+RHwI7CAkSbCjpfWFROGtYYkUlaBVxCT6UXJ4"
        	}"#,
            )?,
        };

        let tx_body = TxBody::<MsgSend> {
            messages: vec1![MsgSend {
                from_address: AccAddress::from_bech32(
                    "cosmos12vrgunwvszgzpykdrqlx3m6puedvcajlxcyw8z",
                )?,
                to_address: AccAddress::from_bech32(
                    "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                )?,
                amount: UnsignedCoins::new(vec![UnsignedCoin {
                    denom: Denom::try_from("uatom".to_string())?,
                    amount: Uint256::from(1u8),
                }])
                .unwrap(),
            }],
            memo: String::new(),
            timeout_height: 0,
            extension_options: Vec::new(),
            non_critical_extension_options: Vec::new(),
        };

        let tx_data = TxData::<MsgSend> {
            body: tx_body,
            auth_info: auth_inf,
        };

        let handler = SignModeHandler;

        let cbor = handler.sign_bytes_get(&TestNoneMetadataGetter, signer_data, tx_data)?;

        let expected = [
            161u8, 1, 142, 162, 1, 104, 67, 104, 97, 105, 110, 32, 105, 100, 2, 106, 116, 101, 115,
            116, 45, 99, 104, 97, 105, 110, 162, 1, 110, 65, 99, 99, 111, 117, 110, 116, 32, 110,
            117, 109, 98, 101, 114, 2, 97, 56, 162, 1, 104, 83, 101, 113, 117, 101, 110, 99, 101,
            2, 97, 54, 163, 1, 103, 65, 100, 100, 114, 101, 115, 115, 2, 120, 45, 99, 111, 115,
            109, 111, 115, 49, 50, 118, 114, 103, 117, 110, 119, 118, 115, 122, 103, 122, 112, 121,
            107, 100, 114, 113, 108, 120, 51, 109, 54, 112, 117, 101, 100, 118, 99, 97, 106, 108,
            120, 99, 121, 119, 56, 122, 4, 245, 163, 1, 106, 80, 117, 98, 108, 105, 99, 32, 107,
            101, 121, 2, 120, 31, 47, 99, 111, 115, 109, 111, 115, 46, 99, 114, 121, 112, 116, 111,
            46, 115, 101, 99, 112, 50, 53, 54, 107, 49, 46, 80, 117, 98, 75, 101, 121, 4, 245, 164,
            1, 99, 75, 101, 121, 2, 120, 82, 48, 51, 66, 50, 32, 54, 48, 68, 49, 32, 54, 56, 51,
            69, 32, 52, 52, 55, 67, 32, 48, 56, 69, 67, 32, 50, 48, 50, 52, 32, 52, 57, 66, 48, 32,
            65, 51, 65, 53, 32, 70, 53, 56, 53, 32, 52, 52, 69, 49, 32, 65, 68, 54, 49, 32, 56, 57,
            49, 52, 32, 57, 53, 65, 48, 32, 53, 53, 67, 52, 32, 50, 52, 70, 65, 32, 53, 49, 55, 50,
            32, 55, 56, 3, 1, 4, 245, 161, 2, 120, 30, 84, 104, 105, 115, 32, 116, 114, 97, 110,
            115, 97, 99, 116, 105, 111, 110, 32, 104, 97, 115, 32, 49, 32, 77, 101, 115, 115, 97,
            103, 101, 163, 1, 109, 77, 101, 115, 115, 97, 103, 101, 32, 40, 49, 47, 49, 41, 2, 120,
            28, 47, 99, 111, 115, 109, 111, 115, 46, 98, 97, 110, 107, 46, 118, 49, 98, 101, 116,
            97, 49, 46, 77, 115, 103, 83, 101, 110, 100, 3, 1, 163, 1, 108, 70, 114, 111, 109, 32,
            97, 100, 100, 114, 101, 115, 115, 2, 120, 45, 99, 111, 115, 109, 111, 115, 49, 50, 118,
            114, 103, 117, 110, 119, 118, 115, 122, 103, 122, 112, 121, 107, 100, 114, 113, 108,
            120, 51, 109, 54, 112, 117, 101, 100, 118, 99, 97, 106, 108, 120, 99, 121, 119, 56,
            122, 3, 2, 163, 1, 106, 84, 111, 32, 97, 100, 100, 114, 101, 115, 115, 2, 120, 45, 99,
            111, 115, 109, 111, 115, 49, 115, 121, 97, 118, 121, 50, 110, 112, 102, 121, 116, 57,
            116, 99, 110, 99, 100, 116, 115, 100, 122, 102, 55, 107, 110, 121, 57, 108, 104, 55,
            55, 55, 112, 97, 104, 117, 117, 120, 3, 2, 163, 1, 102, 65, 109, 111, 117, 110, 116, 2,
            103, 49, 32, 117, 97, 116, 111, 109, 3, 2, 161, 2, 110, 69, 110, 100, 32, 111, 102, 32,
            77, 101, 115, 115, 97, 103, 101, 163, 1, 105, 71, 97, 115, 32, 108, 105, 109, 105, 116,
            2, 103, 50, 48, 48, 39, 48, 48, 48, 4, 245, 163, 1, 113, 72, 97, 115, 104, 32, 111,
            102, 32, 114, 97, 119, 32, 98, 121, 116, 101, 115, 2, 120, 64, 52, 102, 49, 49, 54, 99,
            49, 56, 56, 101, 97, 97, 50, 50, 101, 48, 51, 52, 49, 52, 53, 51, 50, 97, 57, 98, 98,
            51, 57, 49, 54, 102, 102, 53, 53, 98, 97, 49, 99, 55, 48, 56, 100, 49, 97, 100, 97,
            100, 101, 99, 55, 57, 98, 53, 100, 48, 53, 57, 57, 51, 99, 101, 97, 49, 4, 245,
        ];

        assert_eq!(cbor, expected);

        Ok(())
    }

    #[test]
    fn test_sign_bytes_works_v2() -> anyhow::Result<()> {
        let signer_info = SignerInfo {
            public_key: Some(serde_json::from_str(
                r#"{
                        "@type": "/cosmos.crypto.secp256k1.PubKey",
                        "key": "A7Jg0Wg+RHwI7CAkSbCjpfWFROGtYYkUlaBVxCT6UXJ4"
                    }"#,
            )?),
            mode_info: ModeInfo::Single(SignMode::Textual),
            sequence: 13,
        };

        let auth_inf = AuthInfo {
            signer_infos: vec![signer_info],
            fee: Fee {
                amount: None,
                gas_limit: 200_000_u64.try_into().expect("this is a valid gas limit"),
                payer: None,
                granter: String::new(),
            },
            tip: None,
        };

        let signer_data = SignerData {
            address: AccAddress::from_bech32("cosmos12vrgunwvszgzpykdrqlx3m6puedvcajlxcyw8z")?,
            chain_id: ChainId::from_str("test-chain").expect("this is a valid chain id"),
            account_number: 8,
            sequence: 13,
            pub_key: serde_json::from_str(
                r#"{
                    "@type": "/cosmos.crypto.secp256k1.PubKey",
        		"key": "A7Jg0Wg+RHwI7CAkSbCjpfWFROGtYYkUlaBVxCT6UXJ4"
        	}"#,
            )?,
        };

        let tx_body = TxBody::<MsgSend> {
            messages: vec1![MsgSend {
                from_address: AccAddress::from_bech32(
                    "cosmos12vrgunwvszgzpykdrqlx3m6puedvcajlxcyw8z",
                )?,
                to_address: AccAddress::from_bech32(
                    "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux",
                )?,
                amount: UnsignedCoins::new(vec![UnsignedCoin {
                    denom: Denom::try_from("uatom".to_string())?,
                    amount: Uint256::from(1u8),
                }])
                .unwrap(),
            }],
            memo: String::new(),
            timeout_height: 0,
            extension_options: Vec::new(),
            non_critical_extension_options: Vec::new(),
        };

        let tx_data = TxData::<MsgSend> {
            body: tx_body,
            auth_info: auth_inf,
        };

        let handler = SignModeHandler;

        let cbor = handler.sign_bytes_get(&TestNoneMetadataGetter, signer_data, tx_data)?;

        let expected = [
            161, 1, 142, 162, 1, 104, 67, 104, 97, 105, 110, 32, 105, 100, 2, 106, 116, 101, 115,
            116, 45, 99, 104, 97, 105, 110, 162, 1, 110, 65, 99, 99, 111, 117, 110, 116, 32, 110,
            117, 109, 98, 101, 114, 2, 97, 56, 162, 1, 104, 83, 101, 113, 117, 101, 110, 99, 101,
            2, 98, 49, 51, 163, 1, 103, 65, 100, 100, 114, 101, 115, 115, 2, 120, 45, 99, 111, 115,
            109, 111, 115, 49, 50, 118, 114, 103, 117, 110, 119, 118, 115, 122, 103, 122, 112, 121,
            107, 100, 114, 113, 108, 120, 51, 109, 54, 112, 117, 101, 100, 118, 99, 97, 106, 108,
            120, 99, 121, 119, 56, 122, 4, 245, 163, 1, 106, 80, 117, 98, 108, 105, 99, 32, 107,
            101, 121, 2, 120, 31, 47, 99, 111, 115, 109, 111, 115, 46, 99, 114, 121, 112, 116, 111,
            46, 115, 101, 99, 112, 50, 53, 54, 107, 49, 46, 80, 117, 98, 75, 101, 121, 4, 245, 164,
            1, 99, 75, 101, 121, 2, 120, 82, 48, 51, 66, 50, 32, 54, 48, 68, 49, 32, 54, 56, 51,
            69, 32, 52, 52, 55, 67, 32, 48, 56, 69, 67, 32, 50, 48, 50, 52, 32, 52, 57, 66, 48, 32,
            65, 51, 65, 53, 32, 70, 53, 56, 53, 32, 52, 52, 69, 49, 32, 65, 68, 54, 49, 32, 56, 57,
            49, 52, 32, 57, 53, 65, 48, 32, 53, 53, 67, 52, 32, 50, 52, 70, 65, 32, 53, 49, 55, 50,
            32, 55, 56, 3, 1, 4, 245, 161, 2, 120, 30, 84, 104, 105, 115, 32, 116, 114, 97, 110,
            115, 97, 99, 116, 105, 111, 110, 32, 104, 97, 115, 32, 49, 32, 77, 101, 115, 115, 97,
            103, 101, 163, 1, 109, 77, 101, 115, 115, 97, 103, 101, 32, 40, 49, 47, 49, 41, 2, 120,
            28, 47, 99, 111, 115, 109, 111, 115, 46, 98, 97, 110, 107, 46, 118, 49, 98, 101, 116,
            97, 49, 46, 77, 115, 103, 83, 101, 110, 100, 3, 1, 163, 1, 108, 70, 114, 111, 109, 32,
            97, 100, 100, 114, 101, 115, 115, 2, 120, 45, 99, 111, 115, 109, 111, 115, 49, 50, 118,
            114, 103, 117, 110, 119, 118, 115, 122, 103, 122, 112, 121, 107, 100, 114, 113, 108,
            120, 51, 109, 54, 112, 117, 101, 100, 118, 99, 97, 106, 108, 120, 99, 121, 119, 56,
            122, 3, 2, 163, 1, 106, 84, 111, 32, 97, 100, 100, 114, 101, 115, 115, 2, 120, 45, 99,
            111, 115, 109, 111, 115, 49, 115, 121, 97, 118, 121, 50, 110, 112, 102, 121, 116, 57,
            116, 99, 110, 99, 100, 116, 115, 100, 122, 102, 55, 107, 110, 121, 57, 108, 104, 55,
            55, 55, 112, 97, 104, 117, 117, 120, 3, 2, 163, 1, 102, 65, 109, 111, 117, 110, 116, 2,
            103, 49, 32, 117, 97, 116, 111, 109, 3, 2, 161, 2, 110, 69, 110, 100, 32, 111, 102, 32,
            77, 101, 115, 115, 97, 103, 101, 163, 1, 105, 71, 97, 115, 32, 108, 105, 109, 105, 116,
            2, 103, 50, 48, 48, 39, 48, 48, 48, 4, 245, 163, 1, 113, 72, 97, 115, 104, 32, 111,
            102, 32, 114, 97, 119, 32, 98, 121, 116, 101, 115, 2, 120, 64, 48, 56, 56, 56, 48, 97,
            53, 100, 54, 51, 98, 52, 99, 55, 100, 51, 56, 100, 101, 53, 48, 98, 102, 49, 55, 98,
            50, 100, 57, 99, 55, 51, 56, 48, 52, 56, 54, 52, 97, 52, 55, 97, 97, 49, 49, 54, 101,
            56, 101, 50, 99, 57, 51, 52, 51, 48, 101, 54, 55, 57, 53, 97, 102, 56, 4, 245,
        ];

        assert_eq!(cbor, expected);

        Ok(())
    }
}
