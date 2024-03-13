//! Default formatting implementation for bytes - `&[u8]`
use proto_messages::cosmos::tx::v1beta1::screen::Content;
use sha2::{Digest, Sha256};

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, Error, TryPrimitiveValueRenderer,
};

const MAX_BYTE_LENGTH: usize = 35; // Maximum byte length to be displayed as is. Longer than this, we hash.

impl TryPrimitiveValueRenderer<&[u8]> for DefaultPrimitiveRenderer {
    fn try_format(value: &[u8]) -> Result<Content, Error> {
        if value.is_empty() {
            Err(Error::Rendering("cannot render empty bytes".to_string()))
        } else {
            if value.len() <= MAX_BYTE_LENGTH {
                Ok(Content::new(data_encoding::HEXLOWER.encode(value))
                    .expect("value is not empty so it's encoding will not be empty"))
            } else {
                let mut hasher = Sha256::new();

                hasher.update(value);

                let result = hasher.finalize();

                Ok(Content::new(data_encoding::HEXLOWER.encode(&result))
                    .expect("hash is not empty so it's encoding will not be empty"))
            }
        }
    }
}
