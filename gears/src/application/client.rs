use crate::client::{keys, query::run_query, tx::run_tx};

use super::{
    command::client::ClientCommands,
    handlers::{AuxHandler, QueryHandler, TxHandler},
};

/// A Gears client application.
pub trait Client: TxHandler + QueryHandler + AuxHandler {}

pub struct ClientApplication<Core: Client> {
    core: Core,
}

impl<'a, Core: Client> ClientApplication<Core> {
    pub fn new(core: Core) -> Self {
        Self { core }
    }

    /// Runs the command passed
    pub fn execute(
        &self,
        command: ClientCommands<Core::AuxCommands, Core::TxCommands, Core::QueryCommands>,
    ) -> anyhow::Result<()> {
        match command {
            ClientCommands::Aux(cmd) => {
                let cmd = self.core.prepare_aux(cmd)?;
                self.core.handle_aux(cmd)?;
            }
            ClientCommands::Tx(cmd) => run_tx(cmd, &self.core)?,
            ClientCommands::Query(cmd) => run_query(cmd, &self.core)?,
            ClientCommands::Keys(cmd) => keys::keys(cmd)?,
        };

        Ok(())
    }
}
