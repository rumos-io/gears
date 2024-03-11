use crate::{app_v2::ApplicationInfo, client::keys::KeyCommand};

use self::add::CliAddKeyCommand;

pub mod add;

#[derive(Debug, Clone, ::clap::Subcommand)]
#[command(about = "Manage your application's keys")]
pub enum CliKeyCommand<T: ApplicationInfo> {
    Add(CliAddKeyCommand<T>),
}

impl<T: ApplicationInfo> From<CliKeyCommand<T>> for KeyCommand {
    fn from(value: CliKeyCommand<T>) -> Self {
        match value {
            CliKeyCommand::Add(cmd) => KeyCommand::Add(cmd.into()),
        }
    }
}
