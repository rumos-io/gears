use self::{keys::KeyCommand, query::QueryCommand, tx::TxCommand};
use crate::cli::query_txs::{TxQueryCli, TxsQueryCli};

pub mod keys;
pub mod query;
pub mod tx;

#[derive(Debug, Clone)]
pub enum ClientCommands<AUX, TX, QUE> {
    Aux(AUX),
    Tx(TxCommand<TX>),
    Query(QueryCommand<QUE>),
    QueryTx(QueryCommand<TxQueryCli>),
    QueryTxs(QueryCommand<TxsQueryCli>),
    Keys(KeyCommand),
}
