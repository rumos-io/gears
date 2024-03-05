//! Default formatting implementation for `i64`

use num_format::{Buffer, CustomFormat, Grouping};
use once_cell::sync::OnceCell;

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

const THOUSAND_SEPARATOR: &str = "'";

/// Get reference to defined format
pub(super) fn format_get() -> &'static CustomFormat {
    static FORMAT: OnceCell<CustomFormat> = OnceCell::new();

    FORMAT.get_or_init(|| {
        CustomFormat::builder()
            .grouping(Grouping::Standard)
            .minus_sign("-")
            .separator(THOUSAND_SEPARATOR)
            .plus_sign("")
            .build()
            .expect("Failed to build formatter")
    })
}

impl PrimitiveValueRenderer<i64> for DefaultPrimitiveRenderer {
    fn format(value: i64) -> String {
        let mut buf = Buffer::new();
        buf.write_formatted(&value, format_get());

        buf.to_string()
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
            (1_i64, "1"),
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

    #[test]
    fn test_negative() {
        let test_data = [
            (-1_i64, "-1"),
            (-2, "-2"),
            (-10, "-10"),
            (-30, "-30"),
            (-100, "-100"),
            (-500, "-500"),
            (-1000, "-1'000"),
            (-5000, "-5'000"),
            (-10_000, "-10'000"),
            (-100_000, "-100'000"),
            (-5_000_000, "-5'000'000"),
            (-50_000_000, "-50'000'000"),
        ];

        for (i, expected) in test_data {
            let actual = DefaultPrimitiveRenderer::format(i);

            assert_eq!(expected, &actual);
        }
    }
}
