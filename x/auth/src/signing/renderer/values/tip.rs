use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Screen},
    tip::Tip,
    tx_metadata::Metadata,
};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl ValueRenderer for Tip {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let Tip { amount, tipper } = &self;

        if let Some(amount) = amount {
            let mut screens = ValueRenderer::format(amount, get_metadata)?;

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
    use bnum::types::U256;
    use proto_messages::cosmos::{
        base::v1beta1::{Coin, SendCoins},
        tx::v1beta1::{
            screen::{Content, Screen},
            tip::Tip,
        },
    };
    use proto_types::{AccAddress, Denom};

    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };

    #[test]
    fn tip_formating_no_amount() -> anyhow::Result<()> {
        let expected_screens = Vec::<Screen>::new();
        let tip = Tip {
            amount: None,
            tipper: AccAddress::from_bech32("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
        };

        let actuals_screens = ValueRenderer::format(&tip, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }

    #[test]
    fn tip_formating_with_signle_amount() -> anyhow::Result<()> {
        let tip = Tip {
            amount: Some(SendCoins::new(vec![Coin {
                denom: Denom::try_from("uatom".to_owned())?,
                amount: U256::from_digit(2000).into(),
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

        let actuals_screens = ValueRenderer::format(&tip, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }
}
