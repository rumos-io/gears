use serde::de::DeserializeOwned;
use tendermint_abci::cancellation::CancellationSource;
use tendermint_abci::Application;
use tracing::info;

use crate::ext::UnwrapInvalid;
use crate::types::{
    request::{
        begin_block::RequestBeginBlock,
        check_tx::RequestCheckTx,
        deliver_tx::RequestDeliverTx,
        echo::RequestEcho,
        end_block::RequestEndBlock,
        info::RequestInfo,
        init_chain::RequestInitChain,
        query::RequestQuery,
        snapshot::{RequestApplySnapshotChunk, RequestLoadSnapshotChunk, RequestOfferSnapshot},
    },
    response::{
        begin_block::ResponseBeginBlock,
        check_tx::ResponseCheckTx,
        deliver_tx::ResponseDeliverTx,
        echo::ResponseEcho,
        end_block::ResponseEndBlock,
        info::ResponseInfo,
        init_chain::ResponseInitChain,
        query::ResponseQuery,
        snapshot::{
            ResponseApplySnapshotChunk, ResponseListSnapshots, ResponseLoadSnapshotChunk,
            ResponseOfferSnapshot,
        },
        ResponseCommit, ResponseFlush,
    },
};
/// An ABCI application.
///
/// Applications are `Send` + `Clone` + `'static` because they are cloned for
/// each incoming connection to the ABCI [`Server`]. It is up to the
/// application developer to manage shared state between these clones of their
/// application.
///
/// [`Server`]: crate::Server
pub trait ABCIApplication<G>: Send + Clone + 'static {
    /// Echo back the same message as provided in the request.
    fn echo(&self, request: RequestEcho) -> ResponseEcho {
        ResponseEcho {
            message: request.message,
        }
    }

    /// Provide information about the ABCI application.
    fn info(&self, _request: RequestInfo) -> ResponseInfo;

    /// Called once upon genesis.
    fn init_chain(&self, _request: RequestInitChain<G>) -> ResponseInitChain {
        Default::default()
    }

    /// Query the application for data at the current or past height.
    fn query(&self, _request: RequestQuery) -> ResponseQuery;

    /// Check the given transaction before putting it into the local mempool.
    fn check_tx(&self, _request: RequestCheckTx) -> ResponseCheckTx {
        Default::default()
    }

    /// Apply a transaction to the application's state.
    fn deliver_tx(&self, _request: RequestDeliverTx) -> ResponseDeliverTx {
        Default::default()
    }

    /// Signals the beginning of a new block, prior to any `DeliverTx` calls.
    fn begin_block(&self, _request: RequestBeginBlock) -> ResponseBeginBlock {
        Default::default()
    }

    /// Signals the end of a block.
    fn end_block(&self, _request: RequestEndBlock) -> ResponseEndBlock {
        Default::default()
    }

    /// Signals that messages queued on the client should be flushed to the server.
    fn flush(&self) -> ResponseFlush {
        ResponseFlush {}
    }

    /// Commit the current state at the current height.
    fn commit(&self) -> ResponseCommit;

    /// Used during state sync to discover available snapshots on peers.
    fn list_snapshots(&self) -> ResponseListSnapshots {
        Default::default()
    }

    /// Called when bootstrapping the node using state sync.
    fn offer_snapshot(&self, _request: RequestOfferSnapshot) -> ResponseOfferSnapshot {
        Default::default()
    }

    /// Used during state sync to retrieve chunks of snapshots from peers.
    fn load_snapshot_chunk(&self, _request: RequestLoadSnapshotChunk) -> ResponseLoadSnapshotChunk {
        Default::default()
    }

    /// Apply the given snapshot chunk to the application's state.
    fn apply_snapshot_chunk(
        &self,
        _request: RequestApplySnapshotChunk,
    ) -> ResponseApplySnapshotChunk {
        Default::default()
    }
}

#[derive(Debug, Clone)]
pub struct ABCI<T: ABCIApplication<G>, G> {
    handler: T,
    _phantom: std::marker::PhantomData<G>,
}

impl<G, T: ABCIApplication<G>> From<T> for ABCI<T, G> {
    fn from(handler: T) -> Self {
        Self {
            handler,
            _phantom: Default::default(),
        }
    }
}

impl<G: DeserializeOwned + Send + Clone + 'static, T: ABCIApplication<G>> Application
    for ABCI<T, G>
{
    fn echo(
        &self,
        request: tendermint_proto::abci::RequestEcho,
    ) -> tendermint_proto::abci::ResponseEcho {
        info!("Got echo request");

        let guard = CancellationSource::drop_guard();

        let result = T::echo(&self.handler, request.into());

        guard.disarm();
        result.into()
    }

    fn info(
        &self,
        request: tendermint_proto::abci::RequestInfo,
    ) -> tendermint_proto::abci::ResponseInfo {
        info!(
            "Got info request. Tendermint version: {}; Block version: {}; P2P version: {}",
            request.version, request.block_version, request.p2p_version
        );

        let guard = CancellationSource::drop_guard();

        let result = T::info(&self.handler, request.into());

        guard.disarm();
        result.into()
    }

    fn init_chain(
        &self,
        request: tendermint_proto::abci::RequestInitChain,
    ) -> tendermint_proto::abci::ResponseInitChain {
        info!("Got init chain request");

        let guard = CancellationSource::drop_guard();

        let result = T::init_chain(&self.handler, request.try_into().unwrap_or_invalid());

        guard.disarm();
        result.into()
    }

    fn query(
        &self,
        request: tendermint_proto::abci::RequestQuery,
    ) -> tendermint_proto::abci::ResponseQuery {
        info!("Got query request to: {}", request.path);

        /*
           Discuss failure handling for query in future. Maybe we could clean lock poisoning after this method
        */
        let guard = CancellationSource::drop_guard();

        let result = T::query(&self.handler, request.into());

        guard.disarm();
        result.into()
    }

    fn check_tx(
        &self,
        request: tendermint_proto::abci::RequestCheckTx,
    ) -> tendermint_proto::abci::ResponseCheckTx {
        info!("Got check tx request");

        let guard = CancellationSource::drop_guard();

        let result = T::check_tx(&self.handler, request.into());

        guard.disarm();
        result.into()
    }

    fn deliver_tx(
        &self,
        request: tendermint_proto::abci::RequestDeliverTx,
    ) -> tendermint_proto::abci::ResponseDeliverTx {
        info!("Got deliver tx request");

        let guard = CancellationSource::drop_guard();

        let result = T::deliver_tx(&self.handler, request.into());

        guard.disarm();
        result.into()
    }

    fn begin_block(
        &self,
        request: tendermint_proto::abci::RequestBeginBlock,
    ) -> tendermint_proto::abci::ResponseBeginBlock {
        info!("Got begin block request");

        let guard = CancellationSource::drop_guard();

        let result = T::begin_block(&self.handler, request.try_into().unwrap_or_invalid());

        guard.disarm();
        result.into()
    }

    fn end_block(
        &self,
        request: tendermint_proto::abci::RequestEndBlock,
    ) -> tendermint_proto::abci::ResponseEndBlock {
        info!("Got end block request");

        let guard = CancellationSource::drop_guard();

        let result = T::end_block(&self.handler, request.into());

        guard.disarm();
        result.into()
    }

    fn flush(&self) -> tendermint_proto::abci::ResponseFlush {
        info!("Got flush request");

        let guard = CancellationSource::drop_guard();

        let result = T::flush(&self.handler);

        guard.disarm();
        result.into()
    }

    fn commit(&self) -> tendermint_proto::abci::ResponseCommit {
        info!("Got commit request");

        let guard = CancellationSource::drop_guard();

        let result = T::commit(&self.handler);

        guard.disarm();
        result.into()
    }

    fn list_snapshots(&self) -> tendermint_proto::abci::ResponseListSnapshots {
        info!("Got list snapshots request");

        let guard = CancellationSource::drop_guard();

        let result = T::list_snapshots(&self.handler);

        guard.disarm();
        result.into()
    }

    fn offer_snapshot(
        &self,
        request: tendermint_proto::abci::RequestOfferSnapshot,
    ) -> tendermint_proto::abci::ResponseOfferSnapshot {
        info!("Got offer snapshot request");

        let guard = CancellationSource::drop_guard();

        let result = T::offer_snapshot(&self.handler, request.into());

        guard.disarm();
        result.into()
    }

    fn load_snapshot_chunk(
        &self,
        request: tendermint_proto::abci::RequestLoadSnapshotChunk,
    ) -> tendermint_proto::abci::ResponseLoadSnapshotChunk {
        info!("Got load snapshot chunk request");

        let guard = CancellationSource::drop_guard();

        let result = T::load_snapshot_chunk(&self.handler, request.into());

        guard.disarm();
        result.into()
    }

    fn apply_snapshot_chunk(
        &self,
        request: tendermint_proto::abci::RequestApplySnapshotChunk,
    ) -> tendermint_proto::abci::ResponseApplySnapshotChunk {
        info!("Got apply snapshot chunk request");

        let guard = CancellationSource::drop_guard();

        let result = T::apply_snapshot_chunk(&self.handler, request.into());

        guard.disarm();
        result.into()
    }
}
