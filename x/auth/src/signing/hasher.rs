use sha2::Sha256;
use tendermint::informal::crypto::Sha256 as Sha256Diggest;

/// `hash_get` gets the hash of raw bytes to be signed over:
/// HEX(sha256(len(body_bytes) ++ body_bytes ++ len(auth_info_bytes) ++ auth_info_bytes))
pub fn hash_get(body_bytes: &[u8], auth_info_bytes: &[u8]) -> String {
    let mut buffer = Vec::<u8>::with_capacity(16 + body_bytes.len() + auth_info_bytes.len());
    let body_len = (body_bytes.len() as u64).to_be_bytes();
    let auth_info_len = (auth_info_bytes.len() as u64).to_be_bytes();

    buffer.extend(body_len);
    buffer.extend(body_bytes);

    buffer.extend(auth_info_len);
    buffer.extend(auth_info_bytes);

    let finalize = Sha256::digest(buffer);

    hex::encode(finalize)
}
