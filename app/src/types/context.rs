use database::{PrefixDB, DB};

use crate::store::{KVStore, MultiStore, Store};

pub struct Context<'a, T: DB> {
    pub multi_store: &'a mut MultiStore<T>,
    height: u64,
}

impl<'a, T: DB> Context<'a, T> {
    pub fn new(multi_store: &'a mut MultiStore<T>, height: u64) -> Self {
        Context {
            multi_store,
            height,
        }
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: Store) -> &KVStore<PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    /// Fetches a mutable ref to a KVStore from the MultiStore.
    pub fn get_mutable_kv_store(&mut self, store_key: Store) -> &mut KVStore<PrefixDB<T>> {
        return self.multi_store.get_mutable_kv_store(store_key);
    }

    pub fn get_height(&self) -> u64 {
        self.height
    }
}

/// A Context which holds an immutable reference to a MultiStore
pub struct QueryContext<'a, T: DB> {
    pub multi_store: &'a MultiStore<T>,
    height: u64,
}

impl<'a, T: DB> QueryContext<'a, T> {
    pub fn new(multi_store: &'a MultiStore<T>, height: u64) -> Self {
        QueryContext {
            multi_store,
            height,
        }
    }

    ///  Fetches an immutable ref to a KVStore from the MultiStore.
    pub fn get_kv_store(&self, store_key: Store) -> &KVStore<PrefixDB<T>> {
        return self.multi_store.get_kv_store(store_key);
    }

    pub fn get_height(&self) -> u64 {
        self.height
    }
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
