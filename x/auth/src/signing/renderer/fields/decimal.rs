//! Default formating implementation for `Decimal`

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

#[derive(Debug, thiserror::Error)]
#[error("Decimal formatting: {0}")]
pub struct DecimalFormatting(pub String);

#[derive(Debug, Clone)]
pub struct DecimalString<'a>(pub &'a str);

impl PrimitiveValueRenderer<DecimalString<'_>> for DefaultPrimitiveRenderer {
    fn format_try(value: DecimalString<'_>) -> Result<String, Box<dyn std::error::Error>> {
        let value = value.0;

        // *From Cosmos.SDK:*
        // If the decimal doesn't contain a point, we assume it's a value formatted using the legacy
        // `math.Dec`. So we try to parse it as an integer and then convert it to a decimal.
        if value.contains('.') {
            let parts = value.split('.').collect::<Vec<_>>();

            if parts.len() > 2 {
                Err(DecimalFormatting("Found 2 or more dots".to_string()))?
            }

            let parsed_int = parts
                .get(0)
                .ok_or(DecimalFormatting(
                    "Failed to get integer part of decimal".to_string(),
                ))?
                .parse::<i64>()?;

            let formatted_int = DefaultPrimitiveRenderer::format(parsed_int);

            if let Some(dec_part) = parts.get(1) {
                let dec_formatted = dec_part.trim().replace("0", "");

                if dec_formatted.is_empty() {
                    Ok(formatted_int)
                } else {
                    Ok(format!("{formatted_int}.{dec_formatted}"))
                }
            } else {
                Ok(formatted_int)
            }
        } else {
            let is_negative = value.contains('-');
            let value = value.replace("-", "");

            if value.len() > 1 && value.starts_with('0') {
                Err(DecimalFormatting(
                    "Invalid decimal. Fount zero at beginning".to_string(),
                ))?
            }

            if value.len() == 1 && value.starts_with('0') {
                return Ok("0".to_string());
            }

            let count = value.chars().filter(|this| *this == '0').count();

            let mut buf = String::with_capacity(20);
            if is_negative {
                buf.push_str("-0.");
            } else {
                buf.push_str("0.");
            }

            let loop_num = 18 - count;
            for _ in 1..loop_num {
                buf.push('0')
            }

            buf.push_str(&value);

            // If value is `100` for example it adds 2 zero too and they should be removed
            while buf.ends_with('0') {
                let _ = buf.pop();
            }

            Ok(buf)
        }
    }

    /// # Panic
    /// In case if `Decimal is invalid`
    fn format(value: DecimalString<'_>) -> String {
        match Self::format_try(value) {
            Ok(var) => var,
            Err(err) => panic!("{err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::signing::renderer::{
        fields::decimal::DecimalString,
        value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer},
    };

    #[test]
    fn test_positive() {
        let test_data = [
            ("0", "0"),
            ("0.0", "0"),
            ("0.123", "0.123"),
            ("1.0", "1"),
            ("1", "0.000000000000000001"),
            ("2", "0.000000000000000002"),
            ("2.0", "2"),
            ("100", "0.0000000000000001"),
            ("100.0", "100"),
            ("10000", "0.00000000000001"),
            ("10000.0", "10'000"),
            ("10000.95479", "10'000.95479"),
            ("1000000000000", "0.000001"),
            ("1000000000000.0", "1'000'000'000'000"),
            ("1000000000000.5747687678", "1'000'000'000'000.5747687678"),
            ("200000000000000000", "0.2"),
            ("200000000000000000.0", "200'000'000'000'000'000"),
            (
                "200000000000000000.774657647",
                "200'000'000'000'000'000.774657647",
            ),
        ];

        for (i, expected) in test_data {
            let actual = DefaultPrimitiveRenderer::format(DecimalString(i));

            assert_eq!(expected, &actual);
        }
    }

    #[test]
    fn test_negative() {
        let test_data = [
            ("-0", "0"),
            ("-0.0", "0"),
            // ("-0.123", "-0.123"),
            ("-1.0", "-1"),
            ("-1", "-0.000000000000000001"),
            ("-2", "-0.000000000000000002"),
            ("-2.0", "-2"),
            ("-100", "-0.0000000000000001"),
            ("-100.0", "-100"),
            ("-10000", "-0.00000000000001"),
            ("-10000.0", "-10'000"),
            ("-10000.95479", "-10'000.95479"),
            ("-1000000000000", "-0.000001"),
            ("-1000000000000.0", "-1'000'000'000'000"),
            ("-1000000000000.5747687678", "-1'000'000'000'000.5747687678"),
            ("-200000000000000000", "-0.2"),
            ("-200000000000000000.0", "-200'000'000'000'000'000"),
            (
                "-200000000000000000.774657647",
                "-200'000'000'000'000'000.774657647",
            ),
        ];

        for (i, expected) in test_data {
            let actual = DefaultPrimitiveRenderer::format(DecimalString(i));

            assert_eq!(expected, &actual);
        }
    }
}
