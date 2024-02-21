use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};
use bnum::types::U256;
use database::Database;
use gears::types::context::context::Context;
use proto_messages::cosmos::{
    base::v1beta1::Coin,
    tx::v1beta1::{
        screen::{Content, Indent, Screen},
        tx_metadata::Metadata,
    },
};
use store::StoreKey;

impl<SK: StoreKey, DB: Database> ValueRenderer<SK, DB> for Coin {
    /// Format `Coin` into `Screen`.
    fn format(
        &self,
        ctx: &Context<'_, '_, DB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let Metadata {
            display,
            denom_units,
            ..
        } = ctx.metadata_get();

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

                let disp_amount = self.amount.0.clone().div(U256::from_digit(10).pow(power));

                let formated_amount = DefaultPrimitiveRenderer::format(disp_amount);

                let screen = Screen {
                    title: "Amount".to_string(),
                    content: Content::new(format!("{formated_amount} {display}"))?,
                    indent: Some(Indent::new(2)?),
                    expert: false,
                };

                Ok(vec![screen])
            }
            _ => Ok(vec![Screen {
                title: "Amount".to_string(),
                content: Content::new(format!(
                    "{} {display}",
                    DefaultPrimitiveRenderer::format(self.amount.0.clone())
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
        value_renderer::ValueRenderer,
        values::test_mocks::{KeyMock, MockContext},
    };
    use anyhow::Ok;
    use bnum::types::U256;
    use gears::types::context::context::Context;
    use proto_messages::cosmos::{
        base::v1beta1::Coin,
        tx::v1beta1::screen::{Content, Indent, Screen},
    };

    #[test]
    fn coin_formatting() -> anyhow::Result<()> {
        let coin = Coin {
            denom: "uatom".try_into()?,
            amount: U256::from_digit(10000000_u64).into(),
        };

        let expected_screens = Screen {
            title: "Amount".to_string(),
            content: Content::new("10 ATOM".to_string())?,
            indent: Some(Indent::new(2)?),
            expert: false,
        };
        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actual_screen = ValueRenderer::<KeyMock, _>::format(&coin, &context);

        assert!(actual_screen.is_ok(), "Failed to retrieve screens");
        assert_eq!(vec![expected_screens], actual_screen.unwrap());

        Ok(())
    }
}
