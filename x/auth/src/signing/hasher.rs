use bytes::Bytes;

/// `hash_get` gets the hash of raw bytes to be signed over:
/// HEX(sha256(len(body_bytes) ++ body_bytes ++ len(auth_info_bytes) ++ auth_info_bytes))
pub fn hash_get(body_bytes: &Bytes, auth_info_bytes: &Bytes) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    hasher.update(body_bytes.len().to_be_bytes());
    hasher.update(body_bytes);

    hasher.update(auth_info_bytes.len().to_be_bytes());
    hasher.update(auth_info_bytes);

    let finalize = hasher.finalize();

    hex::encode(finalize)
}
