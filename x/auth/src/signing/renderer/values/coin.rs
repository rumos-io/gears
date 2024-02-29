use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};
use proto_messages::cosmos::{
    base::v1beta1::Coin,
    tx::v1beta1::{
        screen::{Content, Indent, Screen},
        tx_metadata::Metadata,
    },
};
use proto_types::Denom;
use proto_types::Uint256;

impl ValueRenderer for Coin {
    /// Format `Coin` into `Screen`.
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let Metadata {
            display,
            denom_units,
            ..
        } = get_metadata(&self.denom).ok_or("metadata not found")?;

        let coin_exp = denom_units.iter().find(|this| this.denom == self.denom);
        let denom_exp = denom_units
            .iter()
            .find(|this| this.denom.as_ref() == display);

        match (coin_exp, denom_exp) {
            (Some(coin_exp), Some(denom_exp)) => {
                let power = match coin_exp.exponent > denom_exp.exponent {
                    true => coin_exp.exponent - denom_exp.exponent,
                    false => denom_exp.exponent - coin_exp.exponent,
                };

                let disp_amount = self.amount.clone() / (Uint256::from(10u32).pow(power));

                let formatted_amount = DefaultPrimitiveRenderer::format(disp_amount);

                let screen = Screen {
                    title: "Amount".to_string(),
                    content: Content::new(format!("{formatted_amount} {display}"))?,
                    indent: Some(Indent::new(2)?),
                    expert: false,
                };

                Ok(vec![screen])
            }
            _ => Ok(vec![Screen {
                title: "Amount".to_string(),
                content: Content::new(format!(
                    "{} {display}",
                    DefaultPrimitiveRenderer::format(self.amount.clone())
                ))?,
                indent: Some(Indent::new(2)?),
                expert: false,
            }]),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };
    use anyhow::Ok;
    use proto_messages::cosmos::{
        base::v1beta1::Coin,
        tx::v1beta1::screen::{Content, Indent, Screen},
    };
    use proto_types::Uint256;

    #[test]
    fn coin_formatting() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(10000000_u64).into(),
        };

        let expected_screens = Screen {
            title: "Amount".to_string(),
            content: Content::new("10 ATOM".to_string())?,
            indent: Some(Indent::new(2)?),
            expert: false,
        };

        let actual_screen = ValueRenderer::format(&coin, &get_metadata);

        assert!(actual_screen.is_ok(), "Failed to retrieve screens");
        assert_eq!(vec![expected_screens], actual_screen.unwrap());

        Ok(())
    }

    #[test]
    fn formatting_small_amounts_works() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: Uint256::from(1u8).into(),
        };

        let expected_screens = Screen {
            title: "Amount".to_string(),
            content: Content::new("1 uatom".to_string())?,
            indent: Some(Indent::new(2)?),
            expert: false,
        };

        let actual_screen = ValueRenderer::format(&coin, &get_metadata);

        assert!(actual_screen.is_ok(), "Failed to retrieve screens");
        assert_eq!(vec![expected_screens], actual_screen.unwrap());

        Ok(())
    }
}
