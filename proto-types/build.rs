use std::{env, error::Error};
fn main() -> Result<(), Box<dyn Error>> {
    let account_address_prefix = env::var("ACCOUNT_ADDRESS_PREFIX").map_err(|_| "ACCOUNT_ADDRESS_PREFIX environment variable must be set. This is best done in a .cargo/config.toml file in the root of your project")?;
    println!(
        "cargo:rustc-env=ACCOUNT_ADDRESS_PREFIX={}",
        account_address_prefix
    );

    //println!("cargo:rerun-if-env-changed=ACCOUNT_ADDRESS_PREFIX"); //not working https://github.com/rust-lang/cargo/issues/10358
    Ok(())
}
