use database::{Database, PrefixDB};
use tendermint_informal::{abci::Event, block::Header};

use store_crate::{KVStore, MultiStore, QueryKVStore, QueryMultiStore, StoreKey};

use crate::error::AppError;

pub struct TxContext<'a, T: Database, SK: StoreKey> {
    multi_store: &'a mut MultiStore<T, SK>,
    height: u64,
    pub events: Vec<Event>,
    header: Header,
    _tx_bytes: Vec<u8>,
}

impl<'a, T: Database, SK: StoreKey> TxContext<'a, T, SK> {
    pub fn new(
        multi_store: &'a mut MultiStore<T, SK>,
        height: u64,
        header: Header,
        tx_bytes: Vec<u8>,
    ) -> Self {
        TxContext {
            multi_store,
            height,
            events: vec![],
            header,
            _tx_bytes: tx_bytes,
        }
    }

    pub fn get_header(&self) -> &Header {
        &self.header
    }

    //TODO: implement From on Context
    pub fn as_any<'b>(&'b mut self) -> Context<'b, 'a, T, SK> {
        Context::TxContext(self)
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    pub fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>> {
        return self.multi_store.get_mutable_kv_store(store_key);
    }

    pub fn get_height(&self) -> u64 {
        self.height
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}

pub struct InitContext<'a, T: Database, SK: StoreKey> {
    pub multi_store: &'a mut MultiStore<T, SK>,
    height: u64,
    pub events: Vec<Event>,
    pub chain_id: String,
}

impl<'a, T: Database, SK: StoreKey> InitContext<'a, T, SK> {
    pub fn new(multi_store: &'a mut MultiStore<T, SK>, height: u64, chain_id: String) -> Self {
        InitContext {
            multi_store,
            height,
            events: vec![],
            chain_id,
        }
    }

    //TODO: implement From on Context
    pub fn as_any<'b>(&'b mut self) -> Context<'b, 'a, T, SK> {
        Context::InitContext(self)
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    pub fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>> {
        return self.multi_store.get_mutable_kv_store(store_key);
    }

    pub fn get_height(&self) -> u64 {
        self.height
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }
}

/// This is used when a method can be used in either a tx or init context
pub enum Context<'a, 'b, T: Database, SK: StoreKey> {
    TxContext(&'a mut TxContext<'b, T, SK>),
    InitContext(&'a mut InitContext<'b, T, SK>),
}

impl<'a, 'b, T: Database, SK: StoreKey> Context<'a, 'b, T, SK> {
    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: &SK) -> &KVStore<PrefixDB<T>> {
        match self {
            Context::TxContext(ctx) => return ctx.get_kv_store(store_key),
            Context::InitContext(ctx) => return ctx.multi_store.get_kv_store(store_key),
        }
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    pub fn get_mutable_kv_store(&mut self, store_key: &SK) -> &mut KVStore<PrefixDB<T>> {
        match self {
            Context::TxContext(ctx) => return ctx.get_mutable_kv_store(store_key),
            Context::InitContext(ctx) => return ctx.multi_store.get_mutable_kv_store(store_key),
        }
    }

    pub fn get_height(&self) -> u64 {
        match self {
            Context::TxContext(ctx) => ctx.height,
            Context::InitContext(ctx) => ctx.height,
        }
    }

    pub fn get_chain_id(&self) -> &str {
        match self {
            Context::TxContext(ctx) => ctx.header.chain_id.as_str(),
            Context::InitContext(ctx) => &ctx.chain_id,
        }
    }

    pub fn push_event(&mut self, event: Event) {
        match self {
            Context::TxContext(ctx) => ctx.push_event(event),
            Context::InitContext(ctx) => ctx.events.push(event),
        };
    }

    pub fn append_events(&mut self, mut events: Vec<Event>) {
        match self {
            Context::TxContext(ctx) => ctx.append_events(events),
            Context::InitContext(ctx) => ctx.events.append(&mut events),
        }
    }
}

pub struct QueryContext<'a, T: Database, SK: StoreKey> {
    pub multi_store: QueryMultiStore<'a, T, SK>,
    //_height: u64,
}

impl<'a, T: Database, SK: StoreKey> QueryContext<'a, T, SK> {
    pub fn new(multi_store: &'a MultiStore<T, SK>, version: u32) -> Result<Self, AppError> {
        let multi_store = QueryMultiStore::new(multi_store, version)
            .map_err(|e| AppError::InvalidRequest(e.to_string()))?;
        Ok(QueryContext {
            multi_store,
            //_height: height,
        })
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: &SK) -> &QueryKVStore<'_, PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    // pub fn _get_height(&self) -> u64 {
    //     self._height
    // }
}

// type Context struct {
// 	ctx           context.Context
// 	ms            MultiStore
// 	header        tmproto.Header
// 	headerHash    tmbytes.HexBytes
// 	chainID       string
// 	txBytes       []byte
// 	logger        log.Logger
// 	voteInfo      []abci.VoteInfo
// 	gasMeter      GasMeter
// 	blockGasMeter GasMeter
// 	checkTx       bool
// 	recheckTx     bool // if recheckTx == true, then checkTx must also be true
// 	minGasPrice   DecCoins
// 	consParams    *abci.ConsensusParams
// 	eventManager  *EventManager
// }
