use database::{Database, PrefixDB};
use gears::types::context::init_context::InitContext;
use proto_messages::cosmos::ibc::types::{
    client_state::{ClientStateCommon, ClientStateExecution, ClientStateValidation},
    path::{ClientConsensusStatePath, ClientStatePath},
    types::Height,
    ClientExecutionContext, ClientValidationContext, CommitmentPrefix, CommitmentRoot,
    ContextError, RawClientId, RawConsensusState, Timestamp,
};
use store::{KVStore, StoreKey};

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
pub struct ClientId(pub String);

impl From<&str> for ClientId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

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

pub struct ClientStore<'a, DB>(pub &'a mut KVStore<PrefixDB<DB>>);

impl<'a, DB> From<&'a mut KVStore<PrefixDB<DB>>> for ClientStore<'a, DB> {
    fn from(value: &'a mut KVStore<PrefixDB<DB>>) -> Self {
        Self(value)
    }
}

pub struct InitContextShim<'a, 'b, DB, SK>(pub &'a mut InitContext<'b, DB, SK>); // TODO: What about using `Cow` so we could have option for owned and reference?

impl<'a, 'b, DB: Database, SK: StoreKey> From<&'a mut InitContext<'b, DB, SK>>
    for InitContextShim<'a, 'b, DB, SK>
{
    fn from(value: &'a mut InitContext<'b, DB, SK>) -> Self {
        Self(value)
    }
}

impl<DB: Database> ClientStateCommon for ClientStore<'_, DB> {
    fn verify_consensus_state(
        &self,
        _consensus_state: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
    ) -> Result<(), proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }

    fn client_type(&self) -> proto_messages::cosmos::ibc::types::ClientType {
        todo!()
    }

    fn latest_height(&self) -> Height {
        todo!()
    }

    fn validate_proof_height(
        &self,
        _proof_height: Height,
    ) -> Result<(), proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }

    fn verify_upgrade_client(
        &self,
        _upgraded_client_state: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
        _upgraded_consensus_state: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
        _proof_upgrade_client: proto_messages::cosmos::ibc::types::RawCommitmentProofBytes,
        _proof_upgrade_consensus_state: proto_messages::cosmos::ibc::types::RawCommitmentProofBytes,
        _root: &CommitmentRoot,
    ) -> Result<(), proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }

    fn verify_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &proto_messages::cosmos::ibc::types::RawCommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: proto_messages::cosmos::ibc::types::path::Path,
        _value: Vec<u8>,
    ) -> Result<(), proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }

    fn verify_non_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &proto_messages::cosmos::ibc::types::RawCommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: proto_messages::cosmos::ibc::types::path::Path,
    ) -> Result<(), proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }
}

impl<'a, 'b, DB: Database, SK: StoreKey> ClientStateValidation<InitContextShim<'a, 'b, DB, SK>>
    for ClientStore<'_, DB>
{
    fn verify_client_message(
        &self,
        _ctx: &InitContextShim<'a, 'b, DB, SK>,
        _client_id: &RawClientId,
        _client_message: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
        _update_kind: &proto_messages::cosmos::ibc::types::types::UpdateKind,
    ) -> Result<(), proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }

    fn check_for_misbehaviour(
        &self,
        _ctx: &InitContextShim<'a, 'b, DB, SK>,
        _client_id: &RawClientId,
        _client_message: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
        _update_kind: &proto_messages::cosmos::ibc::types::types::UpdateKind,
    ) -> Result<bool, proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }

    fn status(
        &self,
        _ctx: &InitContextShim<'a, 'b, DB, SK>,
        _client_id: &RawClientId,
    ) -> Result<
        proto_messages::cosmos::ibc::types::types::Status,
        proto_messages::cosmos::ibc::types::ClientError,
    > {
        todo!()
    }
}

impl<'a, 'b, DB: Database + Sync + Send, SK: StoreKey>
    ClientStateExecution<InitContextShim<'a, 'b, DB, SK>> for ClientStore<'_, DB>
{
    fn initialise(
        &self,
        _ctx: &mut InitContextShim<'a, 'b, DB, SK>,
        _client_id: &RawClientId,
        _consensus_state: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
    ) -> Result<(), proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }

    fn update_state(
        &self,
        _ctx: &mut InitContextShim<'a, 'b, DB, SK>,
        _client_id: &RawClientId,
        _header: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
    ) -> Result<Vec<Height>, proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }

    fn update_state_on_misbehaviour(
        &self,
        _ctx: &mut InitContextShim<'a, 'b, DB, SK>,
        _client_id: &RawClientId,
        _client_message: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
        _update_kind: &proto_messages::cosmos::ibc::types::types::UpdateKind,
    ) -> Result<(), proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }

    fn update_state_on_upgrade(
        &self,
        _ctx: &mut InitContextShim<'a, 'b, DB, SK>,
        _client_id: &RawClientId,
        _upgraded_client_state: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
        _upgraded_consensus_state: proto_messages::cosmos::ibc::protobuf::PrimitiveAny,
    ) -> Result<Height, proto_messages::cosmos::ibc::types::ClientError> {
        todo!()
    }
}

impl<'a, 'b, DB: Database + Sync + Send, SK: StoreKey> ClientExecutionContext
    for InitContextShim<'a, 'b, DB, SK>
{
    type V = InitContextShim<'a, 'b, DB, SK>;

    type AnyClientState = ClientStore<'a, DB>;

    type AnyConsensusState = RawConsensusState;

    fn store_client_state(
        &mut self,
        _client_state_path: ClientStatePath,
        _client_state: Self::AnyClientState,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_consensus_state(
        &mut self,
        _consensus_state_path: ClientConsensusStatePath,
        _consensus_state: Self::AnyConsensusState,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_consensus_state(
        &mut self,
        _consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_update_time(
        &mut self,
        _client_id: RawClientId,
        _height: Height,
        _host_timestamp: Timestamp,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_update_height(
        &mut self,
        _client_id: RawClientId,
        _height: Height,
        _host_height: Height,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_update_time(
        &mut self,
        _client_id: RawClientId,
        _height: Height,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_update_height(
        &mut self,
        _client_id: RawClientId,
        _height: Height,
    ) -> Result<(), ContextError> {
        todo!()
    }
}

impl<DB: Database, SK: StoreKey> ClientValidationContext for InitContextShim<'_, '_, DB, SK> {
    fn client_update_time(
        &self,
        _client_id: &RawClientId,
        _height: &Height,
    ) -> Result<Timestamp, ContextError> {
        todo!()
    }

    fn client_update_height(
        &self,
        _client_id: &RawClientId,
        _height: &Height,
    ) -> Result<Height, ContextError> {
        todo!()
    }
}
