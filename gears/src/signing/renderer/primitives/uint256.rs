//! Default formatting implementation for `Uint256` - 256 bit unsigned integer

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};
use crate::types::rendering::screen::Content;
use cosmwasm_std::Uint256;
use num_bigint::BigUint;
use num_format::WriteFormatted;
use std::str::FromStr;

use super::i64::format_get;

impl PrimitiveValueRenderer<Uint256> for DefaultPrimitiveRenderer {
    fn format(value: Uint256) -> Content {
        let value = BigUint::from_str(&value.to_string())
            .expect("the Uint256 to_string format can always be parsed to a BigUint");

        // Small comment: For this num we required to use heap allocated buffer
        let mut buf = String::new();
        let _ = buf.write_formatted(&value, format_get()); // writing into `String` never fails.
        Content::try_new(buf).expect("String will never be empty")
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Uint256;
    use extensions::testing::UnwrapTesting;

    use crate::signing::renderer::value_renderer::{
        DefaultPrimitiveRenderer, PrimitiveValueRenderer,
    };
    use crate::types::rendering::screen::Content;

    #[test]
    fn test_positive() {
        let test_data = [
            (1_u64, "1"),
            (2, "2"),
            (10, "10"),
            (30, "30"),
            (100, "100"),
            (500, "500"),
            (1000, "1'000"),
            (5000, "5'000"),
            (10_000, "10'000"),
            (100_000, "100'000"),
            (5_000_000, "5'000'000"),
            (50_000_000, "50'000'000"),
        ];

        for (i, expected) in test_data {
            let actual = DefaultPrimitiveRenderer::format(Uint256::from(i));

            assert_eq!(Content::try_new(expected).unwrap_test(), actual);
        }
    }
}
