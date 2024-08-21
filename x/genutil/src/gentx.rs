use std::path::PathBuf;

use gears::application::handlers::client::TxHandler;
use staking::{cli::tx::CreateValidatorCli, CreateValidator};

#[derive(Debug, Clone)]
pub struct GentxTxHandler {
    pub output: PathBuf,
}

impl TxHandler for GentxTxHandler {
    type Message = CreateValidator;

    type TxCommands = CreateValidatorCli;

    fn prepare_tx(
        &self,
        _client_tx_context: &gears::commands::client::tx::ClientTxContext,
        command: Self::TxCommands,
        from_address: gears::types::address::AccAddress,
    ) -> anyhow::Result<gears::types::tx::Messages<Self::Message>> {
        command.clone().try_into_cmd(from_address).map(Into::into)
    }

    fn handle_tx(
        &self,
        _raw_tx: gears::types::tx::raw::TxRaw,
        _node: url::Url,
    ) -> anyhow::Result<gears::application::handlers::client::TxExecutionResult> {
        todo!()
    }
}
