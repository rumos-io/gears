use gears::{
    store::{database::Database, StoreKey},
    types::context::init_context::InitContext,
};

use super::GenesisState;
use gears::store::TransactionalKVStore;
use gears::types::context::TransactionalContext;

const KEY_NEXT_CHANNEL_SEQUENCE: &[u8; 19] = b"nextChannelSequence";

#[derive(Debug, Clone)]
pub struct Keeper<SK> {
    store_key: SK,
}

impl<SK: StoreKey> Keeper<SK> {
    pub fn new(store_key: SK) -> Self {
        Self { store_key }
    }

    pub fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        genesis: GenesisState,
    ) {
        // TODO: the following lines(from ibc-go) have not been implemented yet:
        // for _, channel := range gs.Channels {
        //     ch := types.NewChannel(channel.State, channel.Ordering, channel.Counterparty, channel.ConnectionHops, channel.Version)
        //     k.SetChannel(ctx, channel.PortId, channel.ChannelId, ch)
        // }
        // for _, ack := range gs.Acknowledgements {
        //     k.SetPacketAcknowledgement(ctx, ack.PortId, ack.ChannelId, ack.Sequence, ack.Data)
        // }
        // for _, commitment := range gs.Commitments {
        //     k.SetPacketCommitment(ctx, commitment.PortId, commitment.ChannelId, commitment.Sequence, commitment.Data)
        // }
        // for _, receipt := range gs.Receipts {
        //     k.SetPacketReceipt(ctx, receipt.PortId, receipt.ChannelId, receipt.Sequence)
        // }
        // for _, ss := range gs.SendSequences {
        //     k.SetNextSequenceSend(ctx, ss.PortId, ss.ChannelId, ss.Sequence)
        // }
        // for _, rs := range gs.RecvSequences {
        //     k.SetNextSequenceRecv(ctx, rs.PortId, rs.ChannelId, rs.Sequence)
        // }
        // for _, as := range gs.AckSequences {
        //     k.SetNextSequenceAck(ctx, as.PortId, as.ChannelId, as.Sequence)
        // }

        self.set_next_channel_sequence(ctx, genesis.next_channel_sequence);
    }

    pub fn set_next_channel_sequence<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
        sequence: u64,
    ) {
        let ibc_store = ctx.kv_store_mut(&self.store_key);
        ibc_store.set(KEY_NEXT_CHANNEL_SEQUENCE.to_owned(), sequence.to_be_bytes());
    }
}
