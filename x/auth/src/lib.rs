pub mod submission;
mod abci_handler;
mod client;
mod genesis;
mod keeper;
mod message;
mod params;

pub use abci_handler::*;
pub use client::*;
pub use genesis::*;
pub use keeper::*;
pub use message::*;
pub use params::*;

//

pub fn new_module_addr(module_name: &str) -> gears::types::address::AccAddress {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(module_name);
    // sdk behavior. It gets slice of first 20 bytes from sha256 hash
    let addr_bytes = &hash[..20];

    gears::types::address::AccAddress::try_from(addr_bytes.to_vec())
        .expect("vector of 20 bytes can't produce error because 0 < 20 < MAX_ADDR_LEN")
}
