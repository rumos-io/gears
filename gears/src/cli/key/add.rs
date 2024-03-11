use std::{marker::PhantomData, path::PathBuf};

use clap::{ArgAction, ValueHint};

use crate::{
    app_v2::ApplicationInfo,
    client::keys::{AddKeyCommand, KeyringBackend},
};

#[derive(Debug, Clone, ::clap::Args)]
#[command(
    about = "Add a private key (either newly generated or recovered) saving it to <NAME> file"
)]
pub struct CliAddKeyCommand<T: ApplicationInfo> {
    #[arg(required = true)]
    name: String,
    #[arg(short, long, action = ArgAction::SetTrue, help = "Provide seed phrase to recover existing key instead of creating" )]
    recover: bool,
    #[arg(long, action = ArgAction::Set, value_hint = ValueHint::DirPath, default_value_os_t = T::home_dir(), help = "directory for config and data")]
    home: PathBuf,
    /// select keyring's backend
    #[arg(long = "keyring-backend",  action = ArgAction::Set, default_value_t = KeyringBackend::File )]
    keyring_backend: KeyringBackend,

    #[arg(skip)]
    _marker: PhantomData<T>,
}

impl<T: ApplicationInfo> From<CliAddKeyCommand<T>> for AddKeyCommand {
    fn from(value: CliAddKeyCommand<T>) -> Self {
        let CliAddKeyCommand {
            name,
            recover,
            home,
            keyring_backend,
            _marker,
        } = value;

        Self {
            name,
            recover,
            home,
            keyring_backend,
        }
    }
}
