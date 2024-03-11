use crate::client::{query::run_query_command, tx::run_tx_command};

use super::{
    command::client::ClientCommands,
    handlers::{AuxHandler, QueryHandler, TxHandler},
};

/// A Gears client application.
pub trait ClientTrait: TxHandler + QueryHandler + AuxHandler {}

pub struct ClientApplication<Core: ClientTrait> {
    app_core: Core,
}

impl<'a, Core: ClientTrait> ClientApplication<Core> {
    pub fn new(app_core: Core) -> Self {
        Self { app_core }
    }

    /// Runs the command passed on the command line.
    pub fn execute(
        &self,
        command: ClientCommands<Core::AuxCommands, Core::TxCommands, Core::QueryCommands>,
    ) -> anyhow::Result<()> {
        match command {
            ClientCommands::Aux(cmd) => self.app_core.handle_aux_commands(cmd)?,
            ClientCommands::Tx(cmd) => {
                tokio::runtime::Runtime::new()
                    .expect("unclear why this would ever fail")
                    .block_on(run_tx_command::<Core::Message, _, _>(cmd, &self.app_core))?;
            }
            ClientCommands::Query(cmd) => run_query_command(cmd, &self.app_core)?,
        };

        Ok(())
    }
}
