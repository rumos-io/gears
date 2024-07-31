use self::{keys::KeyCommand, query::QueryCommand, tx::TxCommand};
use crate::cli::query::{TxQueryCli, TxsQueryCli};

pub mod keys;
pub mod query;
pub mod tx;

#[derive(Debug, Clone)]
pub enum ClientCommands<AUX, TX, QUE> {
    Aux(AUX),
    Tx(TxCommand<TX>),
    Query(ExtendedQueryCommand<QUE, TxQueryCli, TxsQueryCli>),
    Keys(KeyCommand),
}

#[derive(Debug, Clone)]
pub enum ExtendedQueryCommand<QUE, TX, TXS> {
    QueryCmd(QueryCommand<QUE>),
    Tx(QueryCommand<TX>),
    Txs(QueryCommand<TXS>),
}
