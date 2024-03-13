use proto_messages::cosmos::{
    ibc::protobuf::Protobuf,
    tx::v1beta1::{
        message::Message,
        screen::{Indent, Screen},
        textual_data::TextualData,
        tx_metadata::Metadata,
    },
};
use proto_types::Denom;

use crate::signing::{
    hasher::hash_get,
    renderer::value_renderer::{
        DefaultPrimitiveRenderer, Error, TryPrimitiveValueRenderer, ValueRenderer,
    },
};

impl<M: Message + ValueRenderer> ValueRenderer for TextualData<M> {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Error> {
        let TextualData {
            body,
            auth_info,
            signer_data,
        } = &self; // we need to remember using all fields

        let messages_count = body.messages.len();

        let mut screens = Vec::<Screen>::new();

        // =========================
        screens.append(&mut ValueRenderer::format(signer_data, get_metadata)?);

        // Transaction message section
        screens.push(Screen {
            title: String::new(),
            content: DefaultPrimitiveRenderer::try_format(match messages_count {
                1 => format!("This transaction has 1 Message"),
                _ => format!("This transaction has {} Messages", body.messages.len()),
            })
            .expect("hard coded Strings are not empty"),
            indent: None,
            expert: false,
        });

        for (i, ms) in body.messages.iter().enumerate() {
            screens.push(Screen {
                title: format!("Message ({}/{messages_count})", i + 1),
                content: DefaultPrimitiveRenderer::try_format(ms.type_url().to_string())?,
                indent: Some(Indent::one()),
                expert: false,
            });
            screens.append(&mut ValueRenderer::format(ms, get_metadata)?);
        }
        screens.push(Screen {
            title: String::new(),
            content: DefaultPrimitiveRenderer::try_format("End of Message".to_string())
                .expect("hard coded String is not empty"),
            indent: None,
            expert: false,
        });
        if let Ok(memo) = DefaultPrimitiveRenderer::try_format(body.memo.clone()) {
            screens.push(Screen {
                title: "Memo".to_string(),
                content: memo,
                indent: None,
                expert: false,
            });
        }

        // =========================
        screens.append(&mut ValueRenderer::format(auth_info, get_metadata)?);

        // =========================
        let body_bytes = body.to_owned().encode_vec();
        let auth_info_bytes = auth_info.to_owned().encode_vec();

        screens.push(Screen {
            title: "Hash of raw bytes".to_string(),
            content: DefaultPrimitiveRenderer::try_format(hash_get(&body_bytes, &auth_info_bytes))?,
            indent: None,
            expert: true,
        });

        Ok(screens)
    }
}

#[cfg(test)]
mod tests {
    use proto_messages::cosmos::tx::v1beta1::mode_info::{ModeInfo, SignMode};
    use proto_messages::cosmos::tx::v1beta1::signer::SignerInfo;
    use proto_messages::cosmos::tx::v1beta1::signer_data::SignerData;
    use proto_messages::cosmos::{
        bank::v1beta1::MsgSend,
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::{
            auth_info::AuthInfo,
            fee::Fee,
            screen::{Content, Indent, Screen},
            textual_data::TextualData,
            tx_body::TxBody,
            tx_data::TxData,
        },
    };
    use proto_types::{AccAddress, Denom, Uint256};
    use tendermint::informal::chain::Id;

    use crate::signing::renderer::value_renderer::ValueRenderer;
    use crate::signing::renderer::values::test_functions::get_metadata;

    #[test]
    fn textual_data_formatting() -> anyhow::Result<()> {
        let data = textual_data_get()?;
        let expected_screens = expected_screens_get()?;

        let actuals_screens = ValueRenderer::format(&data, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        if expected_screens != actuals_screens {
            let expected = serde_json::to_string(&expected_screens)?;
            let actual = serde_json::to_string(&actuals_screens)?;
            panic!("Expected: {expected} \n !=\n Actual: {actual}")
        }

        Ok(())
    }

    fn textual_data_get() -> anyhow::Result<TextualData<MsgSend>> {
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

        let auth_info = AuthInfo {
            signer_infos: vec![signer_info],
            fee: Fee {
                amount: Some(
                    SendCoins::new(vec![Coin {
                        denom: Denom::try_from("uatom".to_owned())?,
                        amount: Uint256::from(2000u32),
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
            address: AccAddress::from_bech32("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
            chain_id: Id::try_from("my-chain".to_string()).expect("this is a valid chain id"),
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
            auth_info: auth_info,
        };

        let data = TextualData::new(signer_data, tx_data);

        Ok(data)
    }

    fn expected_screens_get() -> anyhow::Result<Vec<Screen>> {
        let scrreens = vec![
            Screen {
                title: "Chain id".to_string(),
                content: Content::new("my-chain".to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Account number".to_string(),
                content: Content::new(1.to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Sequence".to_string(),
                content: Content::new(2.to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Address".to_string(),
                content: Content::new("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Public key".to_string(),
                content: Content::new("/cosmos.crypto.secp256k1.PubKey")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Key".to_string(),
                content: Content::new( "02EB DD7F E4FD EB76 DC8A 205E F65D 790C D30E 8A37 5A5C 2528 EB3A 923A F1FB 4D79 4D" )?,
                indent: Some(Indent::one()),
                expert: true,
            },
            Screen {
                title: String::new(),
                content: Content::new("This transaction has 1 Message")?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Message (1/1)".to_string(),
                content: Content::new("/cosmos.bank.v1beta1.MsgSend")?,
                indent: Some(Indent::one()),
                expert: false,
            },
            Screen {
                title: "From address".to_string(),
                content: Content::new("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
                indent: Some(Indent::two()),
                expert: false,
            },
            Screen {
                title: "To address".to_string(),
                content: Content::new("cosmos1ejrf4cur2wy6kfurg9f2jppp2h3afe5h6pkh5t")?,
                indent: Some(Indent::two()),
                expert: false,
            },
            Screen {
                title: "Amount".to_string(),
                content: Content::new("10 ATOM")?,
                indent: Some(Indent::two()),
                expert: false,
            },
            Screen {
                title: String::new(),
                content: Content::new("End of Message")?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Fees".to_string(),
                content: Content::new("0.002 ATOM".to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Gas limit".to_string(),
                content: Content::new("100'000".to_string())?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Hash of raw bytes".to_string(),
                content: Content::new(
                    "785bd306ea8962cdb9600089bdd65f3dc029e1aea112dee69e19546c9adad86e",
                )?,
                indent: None,
                expert: true,
            },
        ];

        Ok(scrreens)
    }
}
