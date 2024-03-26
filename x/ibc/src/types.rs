use crate::keeper::KEY_CLIENT_STORE_PREFIX;
use database::Database;
use gears::types::context::{query_context::QueryContext, tx_context::TxContext};
use proto_messages::{
    any::PrimitiveAny,
    cosmos::ibc::{
        protobuf::Protobuf,
        types::{
            core::{
                channel::{
                    channel::ChannelEnd, packet::Receipt, AcknowledgementCommitment,
                    PacketCommitment,
                },
                client::{
                    context::{types::Height, ClientExecutionContext, ClientValidationContext},
                    error::ClientError,
                },
                commitment::CommitmentPrefix,
                connection::ConnectionEnd,
                handler::{error::ContextError, events::IbcEvent},
                host::{
                    identifiers::{ConnectionId, Sequence},
                    path::{
                        AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath,
                        ClientStatePath, CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath,
                        SeqRecvPath, SeqSendPath,
                    },
                },
            },
            primitives::Timestamp,
            tendermint::{
                consensus_state::WrappedConsensusState, context::CommonContext,
                WrappedTendermintClientState,
            },
        },
    },
};
use store::StoreKey;

// TODO: try to find this const in external crates
pub const ATTRIBUTE_KEY_MODULE: &str = "module";

#[derive(
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Signer(pub String);

impl From<&str> for Signer {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

pub enum IbcContext<'a, 'b, DB, SK> {
    Query(&'a QueryContext<'b, DB, SK>),
    Tx(&'a mut TxContext<'b, DB, SK>),
}

pub struct ContextShim<'a, 'b, DB, SK> {
    pub ctx: &'a mut TxContext<'b, DB, SK>,
    pub store_key: SK,
} // TODO: What about using `Cow` so we could have option for owned and reference? Note: I don't think Cow support mutable borrowing

impl<'a, 'b, DB, SK: StoreKey> ContextShim<'a, 'b, DB, SK> {
    pub fn new(ctx: &'a mut TxContext<'b, DB, SK>, store_key: SK) -> Self {
        Self { ctx, store_key }
    }
}

impl<'a, 'b, DB, SK> From<&'a QueryContext<'b, DB, SK>> for IbcContext<'a, 'b, DB, SK> {
    fn from(value: &'a QueryContext<'b, DB, SK>) -> Self {
        Self::Query(value)
    }
}

impl<'a, 'b, DB, SK> From<&'a mut TxContext<'b, DB, SK>> for IbcContext<'a, 'b, DB, SK> {
    fn from(value: &'a mut TxContext<'b, DB, SK>) -> Self {
        Self::Tx(value)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Infallible")]
pub struct InfallibleError;

impl From<InfallibleError> for ClientError {
    fn from(value: InfallibleError) -> Self {
        ClientError::Other {
            description: value.to_string(),
        }
    }
}

pub struct ConsensusState(pub WrappedConsensusState);

impl TryFrom<ConsensusState> for WrappedConsensusState {
    type Error = InfallibleError;

    fn try_from(value: ConsensusState) -> Result<Self, Self::Error> {
        Ok(value.0)
    }
}

impl<'a, 'b, DB: Database, SK: StoreKey> CommonContext for ContextShim<'a, 'b, DB, SK> {
    type ConversionError = InfallibleError;

    type AnyConsensusState = ConsensusState;

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        todo!()
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        todo!()
    }

    fn consensus_state(
        &self,
        _client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        todo!()
    }

    fn consensus_state_heights(
        &self,
        _client_id: &proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
    ) -> Result<Vec<Height>, ContextError> {
        todo!()
    }
}

impl<'a, 'b, DB: Database + Send + Sync, SK: StoreKey>
    proto_messages::cosmos::ibc::types::core::host::ValidationContext
    for ContextShim<'a, 'b, DB, SK>
{
    type V = Self;

    type E = Self;

    type AnyConsensusState =
        proto_messages::cosmos::ibc::types::tendermint::consensus_state::WrappedConsensusState;

    type AnyClientState = WrappedTendermintClientState;

    fn get_client_validation_context(&self) -> &Self::V {
        todo!()
    }

    fn client_state(
        &self,
        _client_id: &proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
    ) -> Result<Self::AnyClientState, ContextError> {
        todo!()
    }

    fn decode_client_state(
        &self,
        _client_state: PrimitiveAny,
    ) -> Result<Self::AnyClientState, ContextError> {
        todo!()
    }

    fn consensus_state(
        &self,
        _client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        unimplemented!() // TODO: Implement
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        todo!()
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        unimplemented!() // TODO: Implement
    }

    fn host_consensus_state(
        &self,
        _height: &Height,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        todo!()
    }

    fn client_counter(&self) -> Result<u64, ContextError> {
        todo!()
    }

    fn connection_end(&self, _conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        todo!()
    }

    fn validate_self_client(
        &self,
        _client_state_of_host_on_counterparty: PrimitiveAny,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        todo!()
    }

    fn connection_counter(&self) -> Result<u64, ContextError> {
        todo!()
    }

    fn channel_end(&self, _channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        todo!()
    }

    fn get_next_sequence_send(
        &self,
        _seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        todo!()
    }

    fn get_next_sequence_recv(
        &self,
        _seq_recv_path: &SeqRecvPath,
    ) -> Result<Sequence, ContextError> {
        todo!()
    }

    fn get_next_sequence_ack(&self, _seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
        todo!()
    }

    fn get_packet_commitment(
        &self,
        _commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, ContextError> {
        todo!()
    }

    fn get_packet_receipt(&self, _receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
        todo!()
    }

    fn get_packet_acknowledgement(
        &self,
        _ack_path: &AckPath,
    ) -> Result<AcknowledgementCommitment, ContextError> {
        todo!()
    }

    fn channel_counter(&self) -> Result<u64, ContextError> {
        todo!()
    }

    fn max_expected_time_per_block(&self) -> std::time::Duration {
        todo!()
    }

    fn validate_message_signer(
        &self,
        _signer: &proto_messages::cosmos::ibc::types::primitives::Signer,
    ) -> Result<(), ContextError> {
        todo!()
    }
}

impl<'a, 'b, DB: Database + Send + Sync, SK: StoreKey>
    proto_messages::cosmos::ibc::types::core::host::ExecutionContext
    for ContextShim<'a, 'b, DB, SK>
{
    fn get_client_execution_context(&mut self) -> &mut Self::E {
        todo!()
    }

    fn increase_client_counter(&mut self) -> Result<(), ContextError> {
        todo!()
    }

    fn store_connection(
        &mut self,
        _connection_path: &ConnectionPath,
        _connection_end: ConnectionEnd,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_connection_to_client(
        &mut self,
        _client_connection_path: &ClientConnectionPath,
        _conn_id: ConnectionId,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn increase_connection_counter(&mut self) -> Result<(), ContextError> {
        todo!()
    }

    fn store_packet_commitment(
        &mut self,
        _commitment_path: &CommitmentPath,
        _commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_packet_commitment(
        &mut self,
        _commitment_path: &CommitmentPath,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_packet_receipt(
        &mut self,
        _receipt_path: &ReceiptPath,
        _receipt: Receipt,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_packet_acknowledgement(
        &mut self,
        _ack_path: &AckPath,
        _ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_packet_acknowledgement(&mut self, _ack_path: &AckPath) -> Result<(), ContextError> {
        todo!()
    }

    fn store_channel(
        &mut self,
        _channel_end_path: &ChannelEndPath,
        _channel_end: ChannelEnd,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_next_sequence_send(
        &mut self,
        _seq_send_path: &SeqSendPath,
        _seq: Sequence,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_next_sequence_recv(
        &mut self,
        _seq_recv_path: &SeqRecvPath,
        _seq: Sequence,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_next_sequence_ack(
        &mut self,
        _seq_ack_path: &SeqAckPath,
        _seq: Sequence,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn increase_channel_counter(&mut self) -> Result<(), ContextError> {
        todo!()
    }

    fn emit_ibc_event(&mut self, _event: IbcEvent) -> Result<(), ContextError> {
        todo!()
    }

    fn log_message(&mut self, _message: String) -> Result<(), ContextError> {
        todo!()
    }
}

impl<'a, 'b, DB: Database + Send + Sync, SK: StoreKey> ClientExecutionContext
    for ContextShim<'a, 'b, DB, SK>
{
    type V = Self;

    type AnyClientState = WrappedTendermintClientState;

    type AnyConsensusState = WrappedConsensusState;

    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::AnyClientState,
    ) -> Result<(), ContextError> {
        let encoded_bytes =
            <WrappedTendermintClientState as Protobuf<PrimitiveAny>>::encode_vec(client_state);

        self.ctx.get_mutable_kv_store(&self.store_key).set(
            format!("{KEY_CLIENT_STORE_PREFIX}/{}", client_state_path.0).into_bytes(),
            encoded_bytes,
        );

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        _consensus_state_path: ClientConsensusStatePath,
        _consensus_state: Self::AnyConsensusState,
    ) -> Result<(), ContextError> {
        unimplemented!() // TODO: Implement
    }

    fn delete_consensus_state(
        &mut self,
        _consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_update_time(
        &mut self,
        _client_id: proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
        _height: Height,
        _host_timestamp: Timestamp,
    ) -> Result<(), ContextError> {
        unimplemented!() // TODO: Implement
    }

    fn store_update_height(
        &mut self,
        _client_id: proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
        _height: Height,
        _host_height: Height,
    ) -> Result<(), ContextError> {
        unimplemented!() // TODO: Implement
    }

    fn delete_update_time(
        &mut self,
        _client_id: proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
        _height: Height,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_update_height(
        &mut self,
        _client_id: proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
        _height: Height,
    ) -> Result<(), ContextError> {
        todo!()
    }
}

impl<DB: Database, SK: StoreKey> ClientValidationContext for ContextShim<'_, '_, DB, SK> {
    fn client_update_time(
        &self,
        _client_id: &proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
        _height: &Height,
    ) -> Result<Timestamp, ContextError> {
        todo!()
    }

    fn client_update_height(
        &self,
        _client_id: &proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
        _height: &Height,
    ) -> Result<Height, ContextError> {
        todo!()
    }
}

impl<'a, 'b, DB: Database, SK: StoreKey>
    proto_messages::cosmos::ibc::types::tendermint::context::ValidationContext
    for ContextShim<'a, 'b, DB, SK>
{
    fn next_consensus_state(
        &self,
        _client_id: &proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
        _height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        todo!()
    }

    fn prev_consensus_state(
        &self,
        _client_id: &proto_messages::cosmos::ibc::types::core::host::identifiers::ClientId,
        _height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        todo!()
    }
}

pub struct QueryShim<'a, 'b, DB, SK>(pub &'a mut QueryContext<'b, DB, SK>);

// impl<DB : Database, SK : StoreKey> tonic::client::GrpcService<tonic::body::BoxBody> for QueryShim<'_, '_, DB, SK>
// {
//     type ResponseBody = Body<Data = Bytes> + Send + 'static;

//     type Error = StdError;

//     type Future;

//     fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
//         todo!()
//     }

//     fn call(&mut self, request: http::Request<tonic::body::BoxBody>) -> Self::Future {
//         todo!()
//     }
// }
