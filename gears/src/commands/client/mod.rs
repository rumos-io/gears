use self::{keys::KeyCommand, query::QueryCommand, tx::TxCommand};

pub mod keys;
pub mod query;
pub mod tx;

#[derive(Debug, Clone)]
pub enum ClientCommands<AUX, TX, QUE> {
    Aux(AUX),
    Tx(TxCommand<TX>),
    Query(QueryCommand<QUE>),
    Keys(KeyCommand),
}
