use database::Database;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Screen},
    tip::Tip,
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<SK: StoreKey, DB: Database> ValueRenderer<SK, DB> for Tip {
    fn format(
        &self,
        ctx: &Context<'_, '_, DB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let Tip { amount, tipper } = &self;

        if let Some(amount) = amount {
            let mut screens = ValueRenderer::<SK, DB>::format(amount, ctx)?;

            screens.push(Screen {
                title: "Tipper".to_string(),
                content: Content::new(tipper.as_hex())?,
                indent: None,
                expert: false,
            });

            Ok(screens)
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use gears::types::context::context::Context;
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::{
            screen::{Content, Screen},
            tip::Tip,
        },
    };
    use proto_types::{AccAddress, Denom, Uint256};

    use crate::signing::renderer::{value_renderer::ValueRenderer, KeyMock, MockContext};

    #[test]
    fn tip_formating_no_amount() -> anyhow::Result<()> {
        let expected_screens = Vec::<Screen>::new();
        let tip = Tip {
            amount: None,
            tipper: AccAddress::from_bech32("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
        };

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actuals_screens = ValueRenderer::<KeyMock, _>::format(&tip, &context)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }

    #[test]
    fn tip_formating_with_signle_amount() -> anyhow::Result<()> {
        let tip = Tip {
            amount: Some(SendCoins::new(vec![Coin {
                denom: Denom::try_from("uatom".to_owned())?,
                amount: Uint256::from(2000u32),
            }])?),
            tipper: AccAddress::from_bech32("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
        };

        let expected_screens = vec![
            Screen {
                title: "Fees".to_string(),
                content: Content::new("0.002 ATOM".to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Tipper".to_string(),
                content: Content::new(tip.tipper.as_hex())?,
                indent: None,
                expert: false,
            },
        ];

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actuals_screens = ValueRenderer::<KeyMock, _>::format(&tip, &context)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }
}
