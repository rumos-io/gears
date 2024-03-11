use self::{app::AppCommands, client::ClientCommands};

/// An empty AUX command if the user does not want to add auxillary commands.
#[derive(Debug, Clone)]
pub struct NilAuxCommand;

#[derive(Debug, Clone)]
pub enum ApplicationCommands<AUX, TX, QUE> {
    Client(ClientCommands<AUX, TX, QUE>),
    App(AppCommands),
}

pub mod client {
    use crate::client::{query::QueryCommand, tx::TxCommand};

    #[derive(Debug, Clone)]
    pub enum ClientCommands<AUX, TX, QUE> {
        Aux(AUX),
        Tx(TxCommand<TX>),
        Query(QueryCommand<QUE>),
    }
}

pub mod app {
    #[derive(Debug, Clone)]
    pub enum AppCommands {
        Init(crate::client::init::InitCommand),
        Run(crate::baseapp::run::RunCommand),
        Keys(crate::client::keys::KeyCommand),
        GenesisAdd(crate::client::genesis_account::GenesisCommand),
    }
}
