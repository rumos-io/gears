use tendermint_informal::crypto::Sha256 as Sha256Diggest;
use sha2::Sha256;

/// `hash_get` gets the hash of raw bytes to be signed over:
/// HEX(sha256(len(body_bytes) ++ body_bytes ++ len(auth_info_bytes) ++ auth_info_bytes))
pub fn hash_get(body_bytes: &Vec< u8 >, auth_info_bytes: &Vec<u8>) -> String {
    let mut buffer = Vec::<u8>::with_capacity(16 + body_bytes.len() + auth_info_bytes.len());
    let body_len = (body_bytes.len() as u64 ).to_be_bytes();
    let auth_info_len = (auth_info_bytes.len() as u64 ).to_be_bytes();

    // let mut hasher = Sha256::new();

    buffer.extend( body_len );
    buffer.extend( body_bytes );

    buffer.extend( auth_info_len );
    buffer.extend( auth_info_bytes );

    let finalize = Sha256::digest( buffer );

    hex::encode(finalize)
}

// pub fn get_hash(body_bz: &[u8], auth_info_bz: &[u8]) -> String {

//     let body_len = (body_bz.len() as u64).to_be_bytes();
//     let auth_info_len = (auth_info_bz.len() as u64).to_be_bytes();

//     let mut b = Vec::with_capacity(16 + body_bz.len() + auth_info_bz.len());
//     b.extend_from_slice(&body_len);
//     b.extend_from_slice(body_bz);
//     b.extend_from_slice(&auth_info_len);
//     b.extend_from_slice(auth_info_bz);

//     let h = Sha256::digest(&b);

//     let f = h.iter().map(|&byte| format!("{:02x}", byte)).collect();

//     dbg!( &f );

//     f
// }