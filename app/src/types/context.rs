use crate::store::MultiStore;

pub struct Context {
    pub multi_store: MultiStore,
}

impl Context {
    pub fn new(mut multi_store: MultiStore) -> Self {
        Context { multi_store }
    }

    pub fn get_multi_store(&self) -> &MultiStore {
        return &self.multi_store;
    }

    pub fn get_mutable_store(&mut self) -> &mut MultiStore {
        return &mut self.multi_store;
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
