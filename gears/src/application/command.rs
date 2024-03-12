use self::{app::AppCommands, client::ClientCommands};

/// An empty AUX command if the user does not want to add auxillary commands.
#[derive(Debug, Clone)]
pub struct NilAuxCommand;

#[derive(Debug, Clone)]
pub struct NilAux;

#[derive(Debug, Clone)]
pub enum ApplicationCommands<ClientAUX, AppAUX, TX, QUE> {
    Client(ClientCommands<ClientAUX, TX, QUE>),
    App(AppCommands<AppAUX>),
}

pub mod client {
    use crate::client::{query::QueryCommand, tx::TxCommand};

    #[derive(Debug, Clone)]
    pub enum ClientCommands<AUX, TX, QUE> {
        Aux(AUX),
        Tx(TxCommand<TX>),
        Query(QueryCommand<QUE>),
        Keys(crate::client::keys::KeyCommand),
    }
}

pub mod app {
    #[derive(Debug, Clone)]
    pub enum AppCommands<AUX> {
        Init(crate::client::init::InitCommand),
        Run(crate::baseapp::run::RunCommand),
        GenesisAdd(crate::client::genesis_account::GenesisCommand),
        Aux(AUX),
    }
}
