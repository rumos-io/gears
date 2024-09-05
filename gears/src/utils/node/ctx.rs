use kv_store::bank::multi::{ApplicationMultiBank, TransactionMultiBank};
use tendermint::types::proto::header::Header;

use crate::{
    baseapp::{options::NodeOptions, ConsensusParams},
    context::{init::InitContext, tx::TxContext},
    types::gas::{
        kind::{BlockKind, TxKind},
        GasMeter,
    },
};

pub struct ContextOptions {
    height: u32,
    header: Header,
    consensus_params: ConsensusParams,
    gas_meter: GasMeter<TxKind>,
    options: NodeOptions,
}

pub fn build_tx_ctx<'a, DB, SK>(
    multi_store: &'a mut TransactionMultiBank<DB, SK>,
    block_gas_meter: &'a mut GasMeter<BlockKind>,
    opt: impl Into<ContextOptions>,
) -> TxContext<'a, DB, SK> {
    let ContextOptions {
        height,
        header,
        consensus_params,
        gas_meter,
        options,
    } = opt.into();
    TxContext::new(
        multi_store,
        height,
        header,
        consensus_params,
        gas_meter,
        block_gas_meter,
        options,
    )
}

pub fn build_init_ctx<DB, SK>(
    multi_store: &mut ApplicationMultiBank<DB, SK>,
    consensus_params: ConsensusParams,
) -> InitContext<'_, DB, SK> {
    InitContext::new(
        multi_store,
        0,
        tendermint::types::time::timestamp::Timestamp::UNIX_EPOCH,
        tendermint::types::chain_id::ChainId::default(),
        consensus_params,
    )
}
