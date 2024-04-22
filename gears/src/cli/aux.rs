use crate::commands::NilAuxCommand;

#[derive(Debug, Clone, ::clap::Subcommand)]
#[command()]
pub enum CliNilAuxCommand {
    #[command(skip)]
    None,
}

impl TryFrom<CliNilAuxCommand> for NilAuxCommand {
    type Error = anyhow::Error;

    fn try_from(_value: CliNilAuxCommand) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
