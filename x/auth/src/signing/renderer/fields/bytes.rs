//! Default formatting implementation for bytes - `&[u8]`
use sha2::{Digest, Sha256};

use crate::signing::renderer::value_renderer::{DefaultPrimitiveRenderer, PrimitiveValueRenderer};

const MAX_BYTE_LENGTH: usize = 35; // Maximum byte length to be displayed as is. Longer than this, we hash.

impl PrimitiveValueRenderer<&[u8]> for DefaultPrimitiveRenderer {
    fn format(value: &[u8]) -> String {
        if value.len() <= MAX_BYTE_LENGTH {
            data_encoding::HEXLOWER.encode(value)
        } else {
            let mut hasher = Sha256::new();

            hasher.update(value);

            let result = hasher.finalize();

            data_encoding::HEXLOWER.encode(&result)
        }
    }
}
