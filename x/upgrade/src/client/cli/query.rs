use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
pub struct UpgradeQueryCli {
    #[command(subcommand)]
    pub command: UpgradeQueryCliCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum UpgradeQueryCliCommands {
    /// get upgrade plan (if one exists)
    #[command(long_about = "Gets the currently scheduled upgrade plan, if one exists")]
    Plan,
    /// block header for height at which a completed upgrade was applied
    #[command(
        long_about = "If upgrade-name was previously executed on the chain, this returns the header for the block at which it was applied. This helps a client determine which binary was valid over a given range of blocks, as well as more context to understand past migrations."
    )]
    Applied {
        /// name of the applied plan to query for
        name: String,
    },
    /// get the list of module versions
    #[command(
        long_about = "Gets a list of module names and their respective consensus versions. Following the command with a specific module name will return only that module's information."
    )]
    ModuleVersions {
        /// name to query a specific module consensus version from state
        name: Option<String>,
    },
}
