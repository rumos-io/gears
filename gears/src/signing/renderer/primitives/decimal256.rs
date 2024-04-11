use crate::proto_types::Decimal256;
use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};
use crate::types::rendering::screen::Content;

impl PrimitiveValueRenderer<Decimal256> for DefaultPrimitiveRenderer {
    fn format(value: Decimal256) -> Content {
        let int_part = value.to_uint_floor();
        let int_part_content = DefaultPrimitiveRenderer::format(int_part);

        let value = value.to_string();
        let parts = value.split('.').collect::<Vec<_>>();
        let dec_part = parts.get(1); // there will always be an int part before the decimal point

        if let Some(dec_part) = dec_part {
            let formatted_int = int_part_content.into_inner();
            Content::new(format!("{formatted_int}.{dec_part}")).expect("this String is not empty")
        } else {
            int_part_content
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::proto_types::Decimal256;
    use crate::signing::renderer::value_renderer::{
        DefaultPrimitiveRenderer, PrimitiveValueRenderer,
    };
    use crate::types::rendering::screen::Content;
    use std::str::FromStr;

    #[test]
    fn format_decimal256_works() {
        let test_data = [
            ("0.1", "0.1"),
            ("0.000000000000000001", "0.000000000000000001"),
            ("1", "1"),
            ("2", "2"),
            ("1000", "1'000"),
            ("5000", "5'000"),
            ("5000000", "5'000'000"),
            (
                "115792089237316195423570985008687907853269984665640564039457.584007913129639935",
                "115'792'089'237'316'195'423'570'985'008'687'907'853'269'984'665'640'564'039'457.584007913129639935",
            ),
        ];

        for (i, expected) in test_data {
            let actual = DefaultPrimitiveRenderer::format(Decimal256::from_str(i).unwrap());

            assert_eq!(Content::new(expected).unwrap(), actual);
        }
    }
}
