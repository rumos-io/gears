use self::{client::ClientCommands, node::AppCommands};

pub mod client;
pub mod node;

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
