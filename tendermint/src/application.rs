use serde::de::DeserializeOwned;
use tendermint_abci::Application;

use crate::cancellation::{CancellationContext, CancellationToken};
use crate::ext::UnwrapInvalid;
use crate::types::response::check_tx::ResponseCheckTx;
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
pub trait ABCIApplication<G, C: Clone + Send + Sync>: Send + Clone + 'static {
    /// Echo back the same message as provided in the request.
    fn echo(&self, request: RequestEcho) -> ResponseEcho {
        ResponseEcho {
            message: request.message,
        }
    }

    /// Provide information about the ABCI application.
    fn info(&self, _request: RequestInfo) -> ResponseInfo;

    /// Called once upon genesis.
    fn init_chain(
        &self,
        _request: RequestInitChain<G>,
        _token: CancellationToken<C>,
    ) -> ResponseInitChain {
        Default::default()
    }

    /// Query the application for data at the current or past height.
    fn query(&self, _request: RequestQuery, _token: CancellationToken<C>) -> ResponseQuery;

    /// Check the given transaction before putting it into the local mempool.
    fn check_tx(&self, _request: RequestCheckTx, _token: CancellationToken<C>) -> ResponseCheckTx {
        Default::default()
    }

    /// Signals the beginning of a new block, prior to any `DeliverTx` calls.
    fn begin_block(&self, _request: RequestBeginBlock) -> ResponseBeginBlock {
        Default::default()
    }

    /// Apply a transaction to the application's state.
    fn deliver_tx(&self, _request: RequestDeliverTx) -> ResponseDeliverTx {
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
pub struct ABCI<T: ABCIApplication<G, C>, C: Clone + Send + Sync, G> {
    handler: T,
    token: CancellationToken<C>,
    _phantom: std::marker::PhantomData<G>,
}

impl<G, C: Clone + Send + Sync, T: ABCIApplication<G, C>> From<T> for ABCI<T, C, G> {
    fn from(handler: T) -> Self {
        Self {
            handler,
            _phantom: Default::default(),
            token: CancellationToken::new(),
        }
    }
}

impl<
        G: DeserializeOwned + Send + Clone + 'static,
        C: Clone + Send + Sync + 'static,
        T: ABCIApplication<G, C>,
    > Application for ABCI<T, C, G>
{
    fn echo(
        &self,
        request: tendermint_proto::abci::RequestEcho,
    ) -> tendermint_proto::abci::ResponseEcho {
        self.token.panic_if_cancelled();

        T::echo(&self.handler, request.into()).into()
    }

    fn info(
        &self,
        request: tendermint_proto::abci::RequestInfo,
    ) -> tendermint_proto::abci::ResponseInfo {
        self.token.panic_if_cancelled();

        T::info(&self.handler, request.into()).into()
    }

    fn init_chain(
        &self,
        request: tendermint_proto::abci::RequestInitChain,
    ) -> tendermint_proto::abci::ResponseInitChain {
        self.token.panic_if_cancelled();

        let guard = self.token.drop_guard();

        let result = T::init_chain(
            &self.handler,
            request.try_into().unwrap_or_invalid(),
            self.token.clone(),
        );

        guard.disarm();

        result.into()
    }

    fn query(
        &self,
        request: tendermint_proto::abci::RequestQuery,
    ) -> tendermint_proto::abci::ResponseQuery {
        self.token.panic_if_cancelled();
        let guard = self.token.drop_guard();

        let result = T::query(&self.handler, request.into(), self.token.clone());

        guard.disarm();

        result.into()
    }

    fn check_tx(
        &self,
        request: tendermint_proto::abci::RequestCheckTx,
    ) -> tendermint_proto::abci::ResponseCheckTx {
        match self.token.if_cancelled_ctx() {
            Some(ctx) => match ctx {
                CancellationContext::GasOverflow => {
                    ResponseCheckTx::error_with_gas_overflow().into()
                }
                _ => {
                    self.token.panic_if_cancelled();
                    unreachable!()
                }
            },
            None => {
                let guard = self.token.drop_guard();

                let result = T::check_tx(&self.handler, request.into(), self.token.clone());

                guard.disarm();

                result.into()
            }
        }
    }

    fn begin_block(
        &self,
        request: tendermint_proto::abci::RequestBeginBlock,
    ) -> tendermint_proto::abci::ResponseBeginBlock {
        self.token.panic_if_cancelled();
        let guard = self.token.drop_guard();

        let result = T::begin_block(&self.handler, request.try_into().unwrap_or_invalid());

        guard.disarm();

        result.into()
    }

    fn deliver_tx(
        &self,
        request: tendermint_proto::abci::RequestDeliverTx,
    ) -> tendermint_proto::abci::ResponseDeliverTx {
        self.token.panic_if_cancelled();
        let guard = self.token.drop_guard();

        let result = T::deliver_tx(&self.handler, request.into());

        guard.disarm();

        result.into()
    }

    fn end_block(
        &self,
        request: tendermint_proto::abci::RequestEndBlock,
    ) -> tendermint_proto::abci::ResponseEndBlock {
        self.token.panic_if_cancelled();
        let guard = self.token.drop_guard();

        let result = T::end_block(&self.handler, request.into());

        guard.disarm();

        result.into()
    }

    fn flush(&self) -> tendermint_proto::abci::ResponseFlush {
        self.token.panic_if_cancelled();

        T::flush(&self.handler).into()
    }

    fn commit(&self) -> tendermint_proto::abci::ResponseCommit {
        self.token.panic_if_cancelled();
        let guard = self.token.drop_guard();

        let result = T::commit(&self.handler);

        guard.disarm();

        result.into()
    }

    fn list_snapshots(&self) -> tendermint_proto::abci::ResponseListSnapshots {
        self.token.panic_if_cancelled();

        T::list_snapshots(&self.handler).into()
    }

    fn offer_snapshot(
        &self,
        request: tendermint_proto::abci::RequestOfferSnapshot,
    ) -> tendermint_proto::abci::ResponseOfferSnapshot {
        self.token.panic_if_cancelled();

        let result = T::offer_snapshot(&self.handler, request.into());

        result.into()
    }

    fn load_snapshot_chunk(
        &self,
        request: tendermint_proto::abci::RequestLoadSnapshotChunk,
    ) -> tendermint_proto::abci::ResponseLoadSnapshotChunk {
        self.token.panic_if_cancelled();

        let result = T::load_snapshot_chunk(&self.handler, request.into());

        result.into()
    }

    fn apply_snapshot_chunk(
        &self,
        request: tendermint_proto::abci::RequestApplySnapshotChunk,
    ) -> tendermint_proto::abci::ResponseApplySnapshotChunk {
        self.token.panic_if_cancelled();

        let result = T::apply_snapshot_chunk(&self.handler, request.into());

        result.into()
    }
}
