use crate::NilAuxCommand;

#[derive(Debug, Clone, ::clap::Subcommand)]
#[command()]
pub enum CliNilAuxCommand {
    #[command(skip)]
    None,
}

impl From<CliNilAuxCommand> for NilAuxCommand {
    fn from(_value: CliNilAuxCommand) -> Self {
        Self
    }
}
