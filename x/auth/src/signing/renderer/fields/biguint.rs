//! Default formating implementation for `BigUint` - 256 bit unsigned integer

use std::str::FromStr;

use num_bigint::BigUint;
use num_format::WriteFormatted;
use proto_types::Uint256;

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

use super::int::format_get;

impl PrimitiveValueRenderer<Uint256> for DefaultPrimitiveRenderer {
    fn format(value: Uint256) -> String {
        let value = BigUint::from_str(&value.to_string()).expect("Failed to map"); //TODO:

        // Small comment: For this num we required to use heap allocated buffer
        let mut buf = String::new();
        let _ = buf.write_formatted(&value, format_get()); // writing into `String` never fails.
        buf
    }

    fn format_try(value: Uint256) -> Result<String, Box<dyn std::error::Error>> {
        Ok(Self::format(value))
    }
}

#[cfg(test)]
mod tests {
    use proto_types::Uint256;

    use crate::signing::renderer::value_renderer::{
        DefaultPrimitiveRenderer, PrimitiveValueRenderer,
    };

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

            assert_eq!(expected, &actual);
        }
    }
}
