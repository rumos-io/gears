//! Default formatting implementation for bytes - `&[u8]`
use gears::types::rendering::screen::Content;
// use proto_messages::cosmos::tx::v1beta1::screen::Content;
use sha2::{Digest, Sha256};

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, Error, TryPrimitiveValueRenderer,
};

const MAX_BYTE_LENGTH: usize = 35; // Maximum byte length to be displayed as is. Longer than this, we hash.

impl TryPrimitiveValueRenderer<&[u8]> for DefaultPrimitiveRenderer {
    fn try_format(value: &[u8]) -> Result<Content, Error> {
        if value.is_empty() {
            Err(Error::Rendering("cannot render empty bytes".to_string()))
        } else if value.len() <= MAX_BYTE_LENGTH {
            Ok(Content::new(format_bytes(value))
                .expect("value is not empty so it's encoding will not be empty"))
        } else {
            let mut hasher = Sha256::new();
            hasher.update(value);
            let hashed = hasher.finalize();
            let prefixed = format!("SHA-256={}", format_bytes(&hashed));
            Ok(Content::new(prefixed).expect("prefixed is not empty"))
        }
    }
}

fn format_bytes(value: &[u8]) -> String {
    let hex = data_encoding::HEXUPPER.encode(value);

    let mut result = String::new();
    let mut counter = 0;
    for ch in hex.chars() {
        if counter == 4 {
            result.push(' ');
            counter = 0;
        }
        result.push(ch);
        counter += 1;
    }

    result.to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_try_format() {
        let test_cases = vec![
            (
                vec![0u8; 0],
                Err(Error::Rendering("cannot render empty bytes".to_string())),
            ),
            (
                vec![0],
                Ok(Content::new("00").expect("hard coded string is not empty")),
            ),
            (
                vec![102, 111, 111],
                Ok(Content::new("666F 6F").expect("hard coded string is not empty")),
            ),
            (
                vec![102, 111, 111, 98],
                Ok(Content::new("666F 6F62").expect("hard coded string is not empty")),
            ),
            (
                vec![102, 111, 111, 98, 97],
                Ok(Content::new("666F 6F62 61").expect("hard coded string is not empty")),
            ),
            (
                vec![102, 111, 111, 98, 97, 114],
                Ok(Content::new("666F 6F62 6172").expect("hard coded string is not empty")),
            ),
            (
                vec![
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
                ],
                Ok(Content::new("0001 0203 0405 0607 0809 0A0B 0C0D 0E0F 1011 1213 1415 1617 1819 1A1B 1C1D 1E1F 2021 22").expect("hard coded string is not empty")),
            ),
            (
                vec![
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
                ],
                Ok(Content::new("SHA-256=5D7E 2D9B 1DCB C85E 7C89 0036 A2CF 2F9F E7B6 6554 F2DF 08CE C6AA 9C0A 25C9 9C21").expect("hard coded string is not empty")),
            ),
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                DefaultPrimitiveRenderer::try_format(input.as_slice()),
                expected
            );
        }
    }
}
