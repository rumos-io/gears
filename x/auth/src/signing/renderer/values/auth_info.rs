use proto_messages::cosmos::tx::v1beta1::{
    auth_info::AuthInfo,
    screen::{Content, Indent, Screen},
    tx_metadata::Metadata,
};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::{Error, ValueRenderer};

impl ValueRenderer for AuthInfo {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Error> {
        let AuthInfo {
            signer_infos,
            fee,
            tip,
        } = &self;
        let mut final_screens = Vec::<Screen>::new();

        final_screens.append(&mut ValueRenderer::format(fee, get_metadata)?);

        if let Some(tip) = tip {
            final_screens.append(&mut ValueRenderer::format(tip, get_metadata)?);
        }
        // Probably need case for other types of signing
        // TODO: !signer_infos.is_empty()
        if false {
            let signer_count = signer_infos.len();
            final_screens.push(Screen {
                title: "Other signer".to_string(),
                content: Content::new(match signer_count {
                    1 => format!("1 SignerInfo"),
                    _ => format!("{signer_count} SignerInfos"),
                })
                .expect("hard coded string is not empty"),
                indent: None,
                expert: true,
            });

            for (i, info) in signer_infos.iter().enumerate() {
                final_screens.push(Screen {
                    title: format!("Other signer ({}/{signer_count})", i + 1),
                    content: Content::new("SignerInfo object")
                        .expect("hard coded string is not empty"),
                    indent: Some(Indent::one()),
                    expert: true,
                });
                final_screens.append(&mut ValueRenderer::format(info, get_metadata)?);
            }

            final_screens.push(Screen {
                title: String::new(),
                content: Content::new("End of Other signer")
                    .expect("hard coded string is not empty"),
                indent: None,
                expert: true,
            });
        }

        Ok(final_screens)
    }
}

#[cfg(test)]
mod tests {
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::{
            auth_info::AuthInfo,
            fee::Fee,
            mode_info::{ModeInfo, SignMode},
            screen::{Content, Screen},
            signer::SignerInfo,
        },
    };
    use proto_types::{Denom, Uint256};

    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };

    #[test]
    fn auth_info_formatting() -> anyhow::Result<()> {
        let auth_info = AuthInfo {
            signer_infos: vec![SignerInfo {
                public_key: Some(serde_json::from_str(
                    r#"{
                            "@type": "/cosmos.crypto.secp256k1.PubKey",
                            "key": "Auvdf+T963bciiBe9l15DNMOijdaXCUo6zqSOvH7TXlN"
                        }"#,
                )?),
                mode_info: ModeInfo::Single(SignMode::Direct),
                sequence: 2,
            }],
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

        let expected_screens = expected_screens_get()?;

        let actuals_screens = ValueRenderer::format(&auth_info, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }

    fn expected_screens_get() -> anyhow::Result<Vec<Screen>> {
        let result = vec![
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
            // Screen {
            //     title: "Other signer".to_string(),
            //     content: Content::new("1 SignerInfo")?,
            //     indent: None,
            //     expert: true,
            // },
            // Screen {
            //     title: "Other signer (1/1)".to_string(),
            //     content: Content::new("SignerInfo object")?,
            //     indent: Some(Indent::new(1)?),
            //     expert: true,
            // },
            // Screen {
            //     title: "Public key".to_string(),
            //     content: Content::new("02EB DD7F E4FD EB76 DC8A 205E F65D 790C D30E 8A37 5A5C 2528 EB3A 923A F1FB 4D79 4D")?,
            //     indent: None,
            //     expert: true,
            // },
            // Screen {
            //     title: "Sequence".to_string(),
            //     content: Content::new(2.to_string())?,
            //     indent: Some(Indent::new(2)?),
            //     expert: true,
            // },
            // Screen {
            //     title: String::new(),
            //     content: Content::new("End of Other signer")?,
            //     indent: None,
            //     expert: true,
            // },
        ];

        Ok(result)
    }
}
