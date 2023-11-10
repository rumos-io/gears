//! Default formating implementation for `i64`
use crate::signing::renderer::value_renderer::{DefaultRenderer, PrimitiveValueRenderer};

const THOUSAND_SEPARATOR: &str = "'";

impl PrimitiveValueRenderer<i64> for DefaultRenderer {

    fn format(value: i64) -> String {
        let mut value = {
            if value.is_positive()
            {
                value.to_string()
            }
            else { //omit sign char
                value.to_string()[1..].to_string()
            }
        };

        let chars_length = value.chars().count();

        // 1. Less than 4 digits don't need any formatting.
        if chars_length <= 3 {
            return value;
        }

        let mut result = String::new();

        // 2. If the length of v is not a multiple of 3 e.g. 1234 or 12345, to achieve 1'234 or 12'345,
        // we can simply slide to the first mod3 values of v that aren't the multiples of 3 then insert in
        // the thousands separator so in this case: write(12'); then the remaining v will be entirely multiple
        // of 3 hence v = 34*

        let mod3 = chars_length % 3;
        if mod3 != 0 {
            result.push_str(&value[..mod3]);

            value = String::from(&value[mod3..]);

            result.push_str(THOUSAND_SEPARATOR);
        }

        // 3. By this point v is entirely multiples of 3 hence we just insert the separator at every 3 digit.
        


        // for i := 0; i < len(v); i += 3 {
        //     end := i + 3
        //     sb.WriteString(v[i:end])
        //     if end < len(v) {
        //         sb.WriteString(thousandSeparator)
        //     }
        // }

        result
    }
}
