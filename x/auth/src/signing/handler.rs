use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    cbor::Cbor, message::Message, signer_data::SignerData, textual_data::TextualData,
    tx_data::TxData,
};
use store::StoreKey;

use super::{
    errors::SigningErrors,
    renderer::value_renderer::{DefaultValueRenderer, ValueRenderer},
};

#[derive(Debug)]
pub struct SignModeHandler;

impl SignModeHandler {
    pub fn sign_bytes_get<SK: StoreKey>(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
        signer_data: SignerData,
        tx_data: TxData<impl Message + ValueRenderer<DefaultValueRenderer, SK>>,
    ) -> Result<Vec<u8>, SigningErrors> {
        let data = TextualData::new(signer_data, tx_data)
            .map_err(|e| SigningErrors::CustomError(e.to_string()))?;

        let screens = data
            .format(ctx)
            .map_err(|e| SigningErrors::CustomError(e.to_string()))?;

        let mut bytes = Vec::new();

        screens.encode(&mut bytes)?;

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use bnum::types::U256;
    use gears::types::context::context::Context;

    use proto_messages::cosmos::{
        bank::v1beta1::MsgSend,
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::{
            auth_info::AuthInfo,
            fee::Fee,
            signer::SignerInfo,
            signer_data::{ChainId, SignerData},
            tx_body::TxBody,
            tx_data::TxData,
        },
    };
    use proto_types::{AccAddress, Denom};

    use crate::signing::{
        handler::SignModeHandler,
        renderer::{KeyMock, MockContext},
    };

    #[test]
    fn test_sign_bytes() -> anyhow::Result<()> {
        let auth_inf = AuthInfo {
            signer_infos: vec![SignerInfo {
                public_key: Some(serde_json::from_str(
                    r#"{
                            "@type": "/cosmos.crypto.secp256k1.PubKey",
                            "key": "Auvdf+T963bciiBe9l15DNMOijdaXCUo6zqSOvH7TXlN"
                        }"#,
                )?),
                mode_info: None,
                sequence: 2,
            }],
            fee: Fee {
                amount: Some(
                    SendCoins::new(vec![Coin {
                        denom: Denom::try_from("uatom".to_owned())?,
                        amount: U256::from_digit(2000),
                    }])
                    .unwrap(),
                ),
                gas_limit: 100000,
                payer: None,
                granter: String::new(),
            },
            tip: None,
        };

        let signer_data = SignerData {
            address: "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs".to_string(),
            chain_id: ChainId::new("my-chain".to_string())?,
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
            messages: vec![MsgSend {
                from_address: AccAddress::from_bech32(
                    "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
                )?,
                to_address: AccAddress::from_bech32(
                    "cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t",
                )?,
                amount: SendCoins::new(vec![Coin {
                    denom: Denom::try_from("uatom".to_string())?,
                    amount: U256::from_digit(10000000),
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
            body_has_unknown_non_criticals: false,
        };

        let handler = SignModeHandler;

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let cbor = handler.sign_bytes_get(&context, signer_data, tx_data)?;

        const EXPECTED_CBOR : &str = "a1018fa20168436861696e20696402686d792d636861696ea2016e4163636f756e74206e756d626572026131a2016853657175656e6365026132a301674164647265737302782d636f736d6f7331756c6176336873656e7570737771666b77327933737570356b677471776e767161386579687304f5a3016a5075626c6963206b657902781f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657904f5a401634b657902785230324542204444374620453446442045423736204443384120323035452046363544203739304320443330452038413337203541354320323532382045423341203932334120463146422034443739203444030104f5a102781e54686973207472616e73616374696f6e206861732031204d657373616765a3016d4d6573736167652028312f312902781c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e640301a3016c46726f6d206164647265737302782d636f736d6f7331756c6176336873656e7570737771666b77327933737570356b677471776e76716138657968730302a3016a546f206164647265737302782d636f736d6f7331656a726634637572327779366b667572673966326a707070326833616665356836706b6835740302a30166416d6f756e74026731302041544f4d0302a1026e456e64206f66204d657373616765a2016446656573026a302e3030322041544f4da30169476173206c696d697402673130302730303004f5a3017148617368206f66207261772062797465730278403738356264333036656138393632636462393630303038396264643635663364633032396531616561313132646565363965313935343663396164616438366504f5";

        validate_result([(cbor, EXPECTED_CBOR)]);

        Ok(())
    }

    fn validate_result<'a>(value: impl IntoIterator<Item = (Vec<u8>, &'a str)>) {
        for (i, expected) in value {
            let expected = data_encoding::HEXLOWER.decode(expected.as_bytes()).unwrap();
            assert_eq!(i, expected, "{i:02x?} != {expected:02x?}");
        }
    }
}
