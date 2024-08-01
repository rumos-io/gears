//! Default formatting implementation for `u64`

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};
use crate::types::rendering::screen::Content;
use num_format::Buffer;

use super::i64::format_get;

impl PrimitiveValueRenderer<u64> for DefaultPrimitiveRenderer {
    fn format(value: u64) -> Content {
        let mut buf = Buffer::new();
        buf.write_formatted(&value, format_get());

        Content::try_new(buf.to_string()).expect("String will never be empty")
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

            assert_eq!(expected, &actual.into_inner());
        }
    }
}
