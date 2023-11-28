use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    auth_info::AuthInfo,
    screen::{Content, Indent, Screen},
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for AuthInfo {
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let AuthInfo {
            signer_infos,
            fee,
            tip,
        } = &self;
        let mut final_screens = Vec::<Screen>::new();

        final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
            fee, ctx,
        )?);

        if let Some(tip) = tip {
            final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                tip, ctx,
            )?);
        }

        let signer_count = signer_infos.len();
        final_screens.push(Screen {
            title: "Other signer".to_string(),
            content: Content::new(match signer_count {
                1 => format!("1 SignerInfo"),
                _ => format!("{signer_count} SignerInfos"),
            })?,
            indent: None,
            expert: true,
        });

        for (i, info) in signer_infos.iter().enumerate() {
            final_screens.push(Screen {
                title: format!("Other signer ({}/{signer_count})", i + 1),
                content: Content::new("SignerInfo object")?,
                indent: Some(Indent::new(1)?),
                expert: true,
            });
            final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                info, ctx,
            )?);
        }

        final_screens.push(Screen {
            title: String::new(),
            content: Content::new("End of Other signer")?,
            indent: None,
            expert: true,
        });

        Ok(final_screens)
    }
}

#[cfg(test)]
mod tests {
    use bnum::types::U256;
    use gears::types::context::context::Context;
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::{
            auth_info::AuthInfo,
            fee::Fee,
            screen::{Content, Indent, Screen},
            signer::SignerInfo,
        },
    };
    use proto_types::{AccAddress, Denom};

    use crate::signing::renderer::{
        value_renderer::{DefaultValueRenderer, ValueRenderer},
        KeyMock, MockContext,
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

        let expected_screens = expected_screens_get()?;

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actuals_screens =
            ValueRenderer::<DefaultValueRenderer, KeyMock>::format(&auth_info, &context)
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
            Screen {
                title: "Other signer".to_string(),
                content: Content::new("1 SignerInfo")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Other signer (1/1)".to_string(),
                content: Content::new("SignerInfo object")?,
                indent: Some(Indent::new(1)?),
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
                content: Content::new(AccAddress::from_bech32(
                    "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
                )?)?,
                indent: Some(Indent::new(1)?),
                expert: true,
            },
            Screen {
                title: "Sequence".to_string(),
                content: Content::new(2.to_string())?,
                indent: Some(Indent::new(2)?),
                expert: true,
            },
            Screen {
                title: String::new(),
                content: Content::new("End of Other signer")?,
                indent: None,
                expert: true,
            }
        ];

        Ok(result)
    }
}
