//pub type GenesisState = ibc::core::channel::types::proto::v1::GenesisState;

use gears::core::serializers::serialize_number_to_string;
use ibc::core::channel::types::{
    packet::PacketState,
    proto::v1::{IdentifiedChannel, PacketSequence},
};
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

/// GenesisState defines the ibc channel submodule's genesis state.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GenesisState {
    pub channels: Vec<IdentifiedChannel>,
    pub acknowledgements: Vec<PacketState>,
    pub commitments: Vec<PacketState>,
    pub receipts: Vec<PacketState>,
    pub send_sequences: Vec<PacketSequence>,
    pub recv_sequences: Vec<PacketSequence>,
    pub ack_sequences: Vec<PacketSequence>,
    /// the sequence for the next generated channel identifier
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(serialize_with = "serialize_number_to_string")]
    pub next_channel_sequence: u64,
}
