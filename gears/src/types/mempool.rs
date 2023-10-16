use database::Database;
use proto_messages::cosmos::tx::v1beta1::{Message, Tx};
use store_crate::StoreKey;

use super::context::context::Context;

pub struct Error;

pub trait MempoolTrait {
    /// Insert attempts to insert a `Tx` into the app-side mempool returning an error upon failure.
    fn insert_tx<'a, T: Database, SK: StoreKey, M: Message>(
        &self,
        ctx: &Context<T, SK>,
        tx: &Tx<M>,
    ) -> Result<(), Error>;
    // Select returns an Iterator over the app-side mempool. If txs are specified,
    // then they shall be incorporated into the Iterator. The Iterator must
    // closed by the caller.
    // Select(context.Context, [][]byte) Iterator
    /// CountTx returns the number of transactions currently in the mempool.
    fn count_tx(&self) -> usize;
    /// Remove attempts to remove a transaction from the mempool, returning an error upon failure.
    fn remove_tx<M: Message>(&self, tx: &Tx<M>) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct MemPool; //TODO: Discuss real implementation

impl MempoolTrait for MemPool {
    fn insert_tx<'a, T: Database, SK: StoreKey, M: Message>(
        &self,
        _ctx: &Context<T, SK>,
        _tx: &Tx<M>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn count_tx(&self) -> usize {
        todo!()
    }

    fn remove_tx<M: Message>(&self, _tx: &Tx<M>) -> Result<(), Error> {
        todo!()
    }
}
