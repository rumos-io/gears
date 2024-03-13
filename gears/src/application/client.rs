use serde::Serialize;

use crate::client::{keys, query::run_query_v2, tx::run_tx};

use super::{
    command::client::ClientCommands,
    handlers::{AuxHandler, TxHandler},
    handlers_v2::QueryHandler,
};

/// A Gears client application.
pub trait Client: TxHandler + QueryHandler + AuxHandler
where
    <Self::QueryResponse as TryFrom<Self::RawQueryResponse>>::Error: std::fmt::Display,
{
}

pub struct ClientApplication<Core: Client>
where
    <Core::QueryResponse as TryFrom<Core::RawQueryResponse>>::Error: std::fmt::Display,
{
    core: Core,
}

impl<'a, Core: Client> ClientApplication<Core>
where
    <Core::QueryResponse as TryFrom<Core::RawQueryResponse>>::Error: std::fmt::Display,
    Core::QueryResponse: Serialize,
{
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
            ClientCommands::Query(cmd) => run_query_v2(cmd, &self.core)?,
            ClientCommands::Keys(cmd) => keys::keys(cmd)?,
        };

        Ok(())
    }
}
