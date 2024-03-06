use anyhow::Result;
use bip32::Mnemonic;
use clap::ValueEnum;
use strum::Display;
use std::path::PathBuf;
use text_io::read;

const KEYRING_SUB_DIR_FILE: &str = "keyring-file";
const KEYRING_SUB_DIR_TEST: &str = "keyring-test";

#[derive(ValueEnum, Clone, Default, Debug, Display)]
pub enum KeyringBackend {
    #[default]
    File,
    Test,
}

impl KeyringBackend {
    pub fn get_sub_dir(&self) -> &str {
        match self {
            KeyringBackend::File => KEYRING_SUB_DIR_FILE,
            KeyringBackend::Test => KEYRING_SUB_DIR_TEST,
        }
    }

    pub fn to_keyring_backend<'a>(&self, path: &'a PathBuf) -> keyring::Backend<'a> {
        match self {
            KeyringBackend::File => keyring::Backend::File(&path),
            KeyringBackend::Test => keyring::Backend::Test(&path),
        }
    }
}

#[derive(Debug, Clone)]
pub enum KeyCommand {
    Add( AddKeyCommand),
}

#[derive(Debug, Clone, derive_builder::Builder,)]
pub struct AddKeyCommand{
    pub name: String,
    pub recover: bool,
    pub home: PathBuf,
    pub keyring_backend: KeyringBackend,
}

// TODO: remove this cli code
pub fn keys(command: KeyCommand) -> Result<()> {
    match command
    {
        KeyCommand::Add( cmd) => 
        {
            let AddKeyCommand { name, recover, home, keyring_backend } = cmd;

            let keyring_home = home.join(keyring_backend.get_sub_dir());

            let backend = keyring_backend.to_keyring_backend(&keyring_home);

            if recover {
                println!("> Enter your bip39 mnemonic");
                let phrase: String = read!("{}\n");

                let mnemonic = Mnemonic::new(phrase, bip32::Language::English)?;

                keyring::add_key(&name, &mnemonic, keyring::KeyType::Secp256k1, backend)?;
            } else {
                let (mnemonic, key_pair) =
                    keyring::create_key(&name, keyring::KeyType::Secp256k1, backend)?;

                println!("Created key {}\nAddress: {}", name, key_pair.get_address());

                println!("\n**Important** write this mnemonic phrase in a safe place.\nIt is the only way to recover your account.\n");
                println!("{}", mnemonic.phrase());
            }
        },
    }

    Ok(())
}
