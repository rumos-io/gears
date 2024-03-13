use proto_messages::cosmos::tx::v1beta1::{fee::Fee, screen::Screen, tx_metadata::Metadata};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, Error, PrimitiveValueRenderer, TryPrimitiveValueRenderer,
    TryPrimitiveValueRendererWithMetadata, ValueRenderer,
};

impl ValueRenderer for Fee {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Error> {
        let Fee {
            amount,
            gas_limit,
            payer,
            granter,
        } = &self;

        let mut screens = Vec::<Screen>::new();

        if let Some(amount) = amount {
            screens.push(Screen {
                title: "Fees".to_string(),
                content: DefaultPrimitiveRenderer::try_format_with_metadata(
                    amount.to_owned(),
                    get_metadata,
                )?,
                indent: None,
                expert: false,
            });
        }

        if let Some(payer) = payer {
            screens.push(Screen {
                title: "Fee payer".to_string(),
                content: DefaultPrimitiveRenderer::format(payer.to_owned()),
                indent: None,
                expert: true,
            });
        }
        if let Ok(granter) = DefaultPrimitiveRenderer::try_format(granter.to_owned()) {
            screens.push(Screen {
                title: "Fee granter".to_string(),
                content: granter,
                indent: None,
                expert: true,
            });
        }

        screens.push(Screen {
            title: "Gas limit".to_string(),
            content: DefaultPrimitiveRenderer::format(*gas_limit),
            indent: None,
            expert: true,
        });

        Ok(screens)
    }
}

#[cfg(test)]
mod tests {
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::{
            fee::Fee,
            screen::{Content, Screen},
        },
    };
    use proto_types::{AccAddress, Denom, Uint256};

    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };

    #[test]
    fn fee_almost_empty() -> anyhow::Result<()> {
        let fee = Fee {
            amount: None,
            gas_limit: 1,
            payer: None,
            granter: String::new(),
        };

        let expected_screens = vec![Screen {
            title: "Gas limit".to_string(),
            content: Content::new(1.to_string())?,
            indent: None,
            expert: true,
        }];

        let actuals_screens = ValueRenderer::format(&fee, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }

    #[test]
    fn fee_with_amount() -> anyhow::Result<()> {
        let fee = Fee {
            amount: Some(SendCoins::new(vec![Coin {
                denom: Denom::try_from("uatom".to_owned())?,
                amount: Uint256::from(2000u32),
            }])?),
            gas_limit: 1,
            payer: None,
            granter: String::new(),
        };

        let expected_screens = vec![
            Screen {
                title: "Fees".to_string(),
                content: Content::new("0.002 ATOM".to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Gas limit".to_string(),
                content: Content::new(1.to_string())?,
                indent: None,
                expert: true,
            },
        ];

        let actuals_screens = ValueRenderer::format(&fee, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }

    #[test]
    fn fee_with_payer() -> anyhow::Result<()> {
        let fee = Fee {
            amount: Some(SendCoins::new(vec![Coin {
                denom: Denom::try_from("uatom".to_owned())?,
                amount: Uint256::from(2000u32),
            }])?),
            gas_limit: 1,
            payer: Some(AccAddress::from_bech32(
                "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
            )?),
            granter: String::new(),
        };

        let expected_screens = vec![
            Screen {
                title: "Fees".to_string(),
                content: Content::new("0.002 ATOM".to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Fee payer".to_string(),
                content: Content::new(fee.payer.clone().unwrap())?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Gas limit".to_string(),
                content: Content::new(1.to_string())?,
                indent: None,
                expert: true,
            },
        ];

        let actuals_screens = ValueRenderer::format(&fee, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }

    #[test]
    fn fee_with_granter() -> anyhow::Result<()> {
        let fee = Fee {
            amount: Some(SendCoins::new(vec![Coin {
                denom: Denom::try_from("uatom".to_owned())?,
                amount: Uint256::from(2000u32),
            }])?),
            gas_limit: 1,
            payer: Some(AccAddress::from_bech32(
                "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs",
            )?),
            granter: "cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs".to_string(),
        };

        let expected_screens = vec![
            Screen {
                title: "Fees".to_string(),
                content: Content::new("0.002 ATOM".to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Fee payer".to_string(),
                content: Content::new("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs").unwrap(),
                indent: None,
                expert: true,
            },
            Screen {
                title: "Fee granter".to_string(),
                content: Content::new("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Gas limit".to_string(),
                content: Content::new(1.to_string())?,
                indent: None,
                expert: true,
            },
        ];

        let actuals_screens = ValueRenderer::format(&fee, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }
}
