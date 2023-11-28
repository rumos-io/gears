use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    fee::Fee,
    screen::{Content, Screen},
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for Fee {
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let Fee {
            amount,
            gas_limit,
            payer,
            granter,
        } = &self;

        let mut screens = Vec::<Screen>::new();
        if let Some(amount) = amount {
            screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                amount, ctx,
            )?);
        }
        if let Some(payer) = payer {
            screens.push(Screen {
                title: "Fee payer".to_string(),
                content: Content::new(payer.as_hex())?,
                indent: None,
                expert: true,
            });
        }
        if let Ok(granter) = Content::new(granter) {
            screens.push(Screen {
                title: "Fee granter".to_string(),
                content: granter,
                indent: None,
                expert: true,
            });
        }

        screens.push(Screen {
            title: "Gas limit".to_string(),
            content: Content::new(DefaultPrimitiveRenderer::format(*gas_limit))?,
            indent: None,
            expert: true,
        });

        Ok(screens)
    }
}

#[cfg(test)]
mod tests {
    use bnum::types::U256;
    use gears::types::context::context::Context;
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::{
            fee::Fee,
            screen::{Content, Screen},
        },
    };
    use proto_types::{AccAddress, Denom};

    use crate::signing::renderer::{
        value_renderer::{DefaultValueRenderer, ValueRenderer},
        KeyMock, MockContext,
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

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actuals_screens =
            ValueRenderer::<DefaultValueRenderer, KeyMock>::format(&fee, &context)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }

    #[test]
    fn fee_with_amount() -> anyhow::Result<()> {
        let fee = Fee {
            amount: Some(SendCoins::new(vec![Coin {
                denom: Denom::try_from("uatom".to_owned())?,
                amount: U256::from_digit(2000),
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

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actuals_screens =
            ValueRenderer::<DefaultValueRenderer, KeyMock>::format(&fee, &context)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }

    #[test]
    fn fee_with_payer() -> anyhow::Result<()> {
        let fee = Fee {
            amount: Some(SendCoins::new(vec![Coin {
                denom: Denom::try_from("uatom".to_owned())?,
                amount: U256::from_digit(2000),
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
                content: Content::new(fee.payer.clone().unwrap().as_hex())?,
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

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actuals_screens =
            ValueRenderer::<DefaultValueRenderer, KeyMock>::format(&fee, &context)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }

    #[test]
    fn fee_with_granter() -> anyhow::Result<()> {
        let fee = Fee {
            amount: Some(SendCoins::new(vec![Coin {
                denom: Denom::try_from("uatom".to_owned())?,
                amount: U256::from_digit(2000),
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
                content: Content::new(fee.payer.clone().unwrap().as_hex())?,
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

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actuals_screens =
            ValueRenderer::<DefaultValueRenderer, KeyMock>::format(&fee, &context)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }
}
