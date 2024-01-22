//! Default formating implementation for `u64`

use num_format::Buffer;

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

use super::int::format_get;

impl PrimitiveValueRenderer<u64> for DefaultPrimitiveRenderer {
    fn format(value: u64) -> String {
        let mut buf = Buffer::new();
        buf.write_formatted(&value, format_get());

        buf.to_string()
    }

    fn format_try(value: u64) -> Result<String, Box<dyn std::error::Error>> {
        Ok(Self::format(value))
    }
}

#[cfg(test)]
mod tests {
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
            let actual = DefaultPrimitiveRenderer::format(i);

            assert_eq!(expected, &actual);
        }
    }
}
