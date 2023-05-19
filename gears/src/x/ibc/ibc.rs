use database::DB;
use ibc::core::ics26_routing::msgs::MsgEnvelope;
use proto_types::AccAddress;

use crate::{error::AppError, types::Context};

const _KEY_NEXT_CLIENT_SEQUENCE: [u8; 18] = [
    110, 101, 120, 116, 067, 108, 105, 101, 110, 116, 083, 101, 113, 117, 101, 110, 099, 101,
]; // "nextClientSequence"

#[derive(Clone)]
pub struct IBCMsg {
    _msg: MsgEnvelope,
    pub signers: Vec<AccAddress>,
}

impl IBCMsg {
    pub fn _new(msg: MsgEnvelope) -> Result<IBCMsg, AppError> {
        let signers = match &msg {
            MsgEnvelope::Client(msg) => match msg {
                ibc::core::ics02_client::msgs::ClientMsg::CreateClient(msg) => {
                    vec![AccAddress::from_bech32(&msg.signer.to_string())
                        .map_err(|e| AppError::IBC(e.to_string()))?]
                }
                ibc::core::ics02_client::msgs::ClientMsg::UpdateClient(_) => todo!(),
                ibc::core::ics02_client::msgs::ClientMsg::Misbehaviour(_) => todo!(),
                ibc::core::ics02_client::msgs::ClientMsg::UpgradeClient(_) => todo!(),
            },
            MsgEnvelope::Connection(_) => todo!(),
            MsgEnvelope::Channel(_) => todo!(),
            MsgEnvelope::Packet(_) => todo!(),
        };

        Ok(IBCMsg { _msg: msg, signers })
    }
}

pub fn _run_tx<'a, 'b, T: DB>(
    _ctx: &'a mut Context<'b, '_, T>,
    _msg: IBCMsg,
) -> Result<(), AppError> {
    //let mut _ibc_context = IBCExecutionContext { _app_ctx: ctx };
    // ibc_context
    //     .execute(msg.msg)
    //     .map_err(|e| AppError::IBC(e.to_string()))
    Ok(())
}

// struct IBCExecutionContext<'a, 'b, T: DB> {
//     _app_ctx: &'a mut Context<'b, T>,
// }

// impl<'a, 'b, T: DB> Router for IBCExecutionContext<'a, 'b, T> {
//     fn get_route(
//         &self,
//         module_id: &ibc::core::ics26_routing::context::ModuleId,
//     ) -> Option<&dyn ibc::core::ics26_routing::context::Module> {
//         todo!()
//     }

//     fn get_route_mut(
//         &mut self,
//         module_id: &ibc::core::ics26_routing::context::ModuleId,
//     ) -> Option<&mut dyn ibc::core::ics26_routing::context::Module> {
//         todo!()
//     }

//     fn has_route(&self, module_id: &ibc::core::ics26_routing::context::ModuleId) -> bool {
//         todo!()
//     }

//     fn lookup_module_by_port(
//         &self,
//         port_id: &ibc::core::ics24_host::identifier::PortId,
//     ) -> Option<ibc::core::ics26_routing::context::ModuleId> {
//         todo!()
//     }
// }

// impl<'a, 'b, T: DB> ValidationContext for IBCExecutionContext<'a, 'b, T> {
//     fn client_state(
//         &self,
//         client_id: &ibc::core::ics24_host::identifier::ClientId,
//     ) -> Result<Box<dyn ibc::core::ics02_client::client_state::ClientState>, ibc::core::ContextError>
//     {
//         todo!()
//     }

//     fn decode_client_state(
//         &self,
//         client_state: ibc_proto::google::protobuf::Any,
//     ) -> Result<Box<dyn ibc::core::ics02_client::client_state::ClientState>, ibc::core::ContextError>
//     {
//         todo!()
//     }

//     fn consensus_state(
//         &self,
//         client_cons_state_path: &ibc::core::ics24_host::path::ClientConsensusStatePath,
//     ) -> Result<
//         Box<dyn ibc::core::ics02_client::consensus_state::ConsensusState>,
//         ibc::core::ContextError,
//     > {
//         todo!()
//     }

//     fn next_consensus_state(
//         &self,
//         client_id: &ibc::core::ics24_host::identifier::ClientId,
//         height: &ibc::Height,
//     ) -> Result<
//         Option<Box<dyn ibc::core::ics02_client::consensus_state::ConsensusState>>,
//         ibc::core::ContextError,
//     > {
//         todo!()
//     }

//     fn prev_consensus_state(
//         &self,
//         client_id: &ibc::core::ics24_host::identifier::ClientId,
//         height: &ibc::Height,
//     ) -> Result<
//         Option<Box<dyn ibc::core::ics02_client::consensus_state::ConsensusState>>,
//         ibc::core::ContextError,
//     > {
//         todo!()
//     }

//     fn host_height(&self) -> Result<ibc::Height, ibc::core::ContextError> {
//         todo!()
//     }

//     fn host_timestamp(&self) -> Result<ibc::timestamp::Timestamp, ibc::core::ContextError> {
//         todo!()
//     }

//     fn host_consensus_state(
//         &self,
//         height: &ibc::Height,
//     ) -> Result<
//         Box<dyn ibc::core::ics02_client::consensus_state::ConsensusState>,
//         ibc::core::ContextError,
//     > {
//         todo!()
//     }

//     fn client_counter(&self) -> Result<u64, ibc::core::ContextError> {
//         todo!()
//     }

//     fn connection_end(
//         &self,
//         conn_id: &ibc::core::ics24_host::identifier::ConnectionId,
//     ) -> Result<ibc::core::ics03_connection::connection::ConnectionEnd, ibc::core::ContextError>
//     {
//         todo!()
//     }

//     fn validate_self_client(
//         &self,
//         client_state_of_host_on_counterparty: ibc_proto::google::protobuf::Any,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn commitment_prefix(&self) -> ibc::core::ics23_commitment::commitment::CommitmentPrefix {
//         todo!()
//     }

//     fn connection_counter(&self) -> Result<u64, ibc::core::ContextError> {
//         todo!()
//     }

//     fn channel_end(
//         &self,
//         channel_end_path: &ibc::core::ics24_host::path::ChannelEndPath,
//     ) -> Result<ibc::core::ics04_channel::channel::ChannelEnd, ibc::core::ContextError> {
//         todo!()
//     }

//     fn get_next_sequence_send(
//         &self,
//         seq_send_path: &ibc::core::ics24_host::path::SeqSendPath,
//     ) -> Result<ibc::core::ics04_channel::packet::Sequence, ibc::core::ContextError> {
//         todo!()
//     }

//     fn get_next_sequence_recv(
//         &self,
//         seq_recv_path: &ibc::core::ics24_host::path::SeqRecvPath,
//     ) -> Result<ibc::core::ics04_channel::packet::Sequence, ibc::core::ContextError> {
//         todo!()
//     }

//     fn get_next_sequence_ack(
//         &self,
//         seq_ack_path: &ibc::core::ics24_host::path::SeqAckPath,
//     ) -> Result<ibc::core::ics04_channel::packet::Sequence, ibc::core::ContextError> {
//         todo!()
//     }

//     fn get_packet_commitment(
//         &self,
//         commitment_path: &ibc::core::ics24_host::path::CommitmentPath,
//     ) -> Result<ibc::core::ics04_channel::commitment::PacketCommitment, ibc::core::ContextError>
//     {
//         todo!()
//     }

//     fn get_packet_receipt(
//         &self,
//         receipt_path: &ibc::core::ics24_host::path::ReceiptPath,
//     ) -> Result<ibc::core::ics04_channel::packet::Receipt, ibc::core::ContextError> {
//         todo!()
//     }

//     fn get_packet_acknowledgement(
//         &self,
//         ack_path: &ibc::core::ics24_host::path::AckPath,
//     ) -> Result<
//         ibc::core::ics04_channel::commitment::AcknowledgementCommitment,
//         ibc::core::ContextError,
//     > {
//         todo!()
//     }

//     fn client_update_time(
//         &self,
//         client_id: &ibc::core::ics24_host::identifier::ClientId,
//         height: &ibc::Height,
//     ) -> Result<ibc::timestamp::Timestamp, ibc::core::ContextError> {
//         todo!()
//     }

//     fn client_update_height(
//         &self,
//         client_id: &ibc::core::ics24_host::identifier::ClientId,
//         height: &ibc::Height,
//     ) -> Result<ibc::Height, ibc::core::ContextError> {
//         todo!()
//     }

//     fn channel_counter(&self) -> Result<u64, ibc::core::ContextError> {
//         todo!()
//     }

//     fn max_expected_time_per_block(&self) -> std::time::Duration {
//         todo!()
//     }
// }

// impl<'a, 'b, T: DB> ExecutionContext for IBCExecutionContext<'a, 'b, T> {
//     fn store_client_type(
//         &mut self,
//         client_type_path: ibc::core::ics24_host::path::ClientTypePath,
//         client_type: ibc::core::ics02_client::client_type::ClientType,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_client_state(
//         &mut self,
//         client_state_path: ibc::core::ics24_host::path::ClientStatePath,
//         client_state: Box<dyn ibc::core::ics02_client::client_state::ClientState>,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_consensus_state(
//         &mut self,
//         consensus_state_path: ibc::core::ics24_host::path::ClientConsensusStatePath,
//         consensus_state: Box<dyn ibc::core::ics02_client::consensus_state::ConsensusState>,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn increase_client_counter(&mut self) {
//         todo!()
//     }

//     fn store_update_time(
//         &mut self,
//         client_id: ibc::core::ics24_host::identifier::ClientId,
//         height: ibc::Height,
//         timestamp: ibc::timestamp::Timestamp,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_update_height(
//         &mut self,
//         client_id: ibc::core::ics24_host::identifier::ClientId,
//         height: ibc::Height,
//         host_height: ibc::Height,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_connection(
//         &mut self,
//         connection_path: &ibc::core::ics24_host::path::ConnectionPath,
//         connection_end: ibc::core::ics03_connection::connection::ConnectionEnd,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_connection_to_client(
//         &mut self,
//         client_connection_path: &ibc::core::ics24_host::path::ClientConnectionPath,
//         conn_id: ibc::core::ics24_host::identifier::ConnectionId,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn increase_connection_counter(&mut self) {
//         todo!()
//     }

//     fn store_packet_commitment(
//         &mut self,
//         commitment_path: &ibc::core::ics24_host::path::CommitmentPath,
//         commitment: ibc::core::ics04_channel::commitment::PacketCommitment,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn delete_packet_commitment(
//         &mut self,
//         commitment_path: &ibc::core::ics24_host::path::CommitmentPath,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_packet_receipt(
//         &mut self,
//         receipt_path: &ibc::core::ics24_host::path::ReceiptPath,
//         receipt: ibc::core::ics04_channel::packet::Receipt,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_packet_acknowledgement(
//         &mut self,
//         ack_path: &ibc::core::ics24_host::path::AckPath,
//         ack_commitment: ibc::core::ics04_channel::commitment::AcknowledgementCommitment,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn delete_packet_acknowledgement(
//         &mut self,
//         ack_path: &ibc::core::ics24_host::path::AckPath,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_channel(
//         &mut self,
//         channel_end_path: &ibc::core::ics24_host::path::ChannelEndPath,
//         channel_end: ibc::core::ics04_channel::channel::ChannelEnd,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_next_sequence_send(
//         &mut self,
//         seq_send_path: &ibc::core::ics24_host::path::SeqSendPath,
//         seq: ibc::core::ics04_channel::packet::Sequence,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_next_sequence_recv(
//         &mut self,
//         seq_recv_path: &ibc::core::ics24_host::path::SeqRecvPath,
//         seq: ibc::core::ics04_channel::packet::Sequence,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn store_next_sequence_ack(
//         &mut self,
//         seq_ack_path: &ibc::core::ics24_host::path::SeqAckPath,
//         seq: ibc::core::ics04_channel::packet::Sequence,
//     ) -> Result<(), ibc::core::ContextError> {
//         todo!()
//     }

//     fn increase_channel_counter(&mut self) {
//         todo!()
//     }

//     fn emit_ibc_event(&mut self, event: ibc::events::IbcEvent) {
//         todo!()
//     }

//     fn log_message(&mut self, message: String) {
//         todo!()
//     }
// }
