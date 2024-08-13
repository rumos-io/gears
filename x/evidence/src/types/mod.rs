use gears::{
    context::TransactionalContext,
    core::{any::google::Any, errors::CoreError},
    derive::Protobuf,
    store::{database::Database, StoreKey},
    tendermint::{
        informal::{hash::Algorithm, Hash},
        types::{
            proto::{info::Evidence as TmEvidence, validator::VotingPower},
            time::timestamp::{Nanoseconds, Timestamp, TimestampSeconds},
        },
    },
    types::address::ConsAddress,
};
use prost::Message;
use serde::{Deserialize, Serialize};

mod query;
mod tx;

// DoubleSignJailEndTime period ends at Max Time supported by Amino
// (Dec 31, 9999 - 23:59:59 GMT).
pub(crate) const DOUBLE_SIGN_JAIL_END_TIME: Timestamp =
    Timestamp::new(TimestampSeconds::MAX, Nanoseconds::MIN);

//

pub trait Evidence: Message + TryFrom<Any> {
    type Error;
    // TODO: uncomment or remove
    // fn route(&self) -> String;
    // Original method is named `type`, replaced as inner interface
    fn kind(&self) -> String;
    fn string(&self) -> String;
    fn hash(&self) -> Hash;
    /// Height at which the infraction occurred
    fn height(&self) -> i64;
    fn handle<CTX: TransactionalContext<DB, SK>, DB: Database, SK: StoreKey>(
        ctx: &mut CTX,
        evidence: &Self,
    ) -> Result<(), <Self as Evidence>::Error>;
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct RawEquivocation {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(message, optional, tag = "2")]
    pub time: Option<Timestamp>,
    #[prost(uint64, tag = "3")]
    pub power: u64,
    #[prost(string, tag = "4")]
    pub consensus_address: String,
}

/// Equivocation implements the Evidence interface and defines evidence of double
/// signing misbehavior.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Protobuf)]
#[proto(raw = "RawEquivocation")]
pub struct Equivocation {
    pub height: i64,
    #[proto(name = "time", optional)]
    pub time: Timestamp,
    pub power: VotingPower,
    pub consensus_address: ConsAddress,
}

impl From<TmEvidence> for Equivocation {
    fn from(
        TmEvidence {
            r#type: _,
            validator,
            height,
            time,
            total_voting_power: _,
        }: TmEvidence,
    ) -> Self {
        Self {
            height,
            time,
            power: validator.power,
            consensus_address: validator.address.into(),
        }
    }
}

impl TryFrom<Any> for RawEquivocation {
    type Error = CoreError;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            // TODO: url?
            "equivocation" => RawEquivocation::decode::<prost::bytes::Bytes>(value.value.into())
                .map_err(|e| gears::core::errors::CoreError::DecodeGeneral(e.to_string())),
            _ => Err(gears::core::errors::CoreError::DecodeGeneral(
                "message type not recognized".into(),
            )),
        }
    }
}

impl Evidence for RawEquivocation {
    type Error = anyhow::Error;

    fn kind(&self) -> String {
        "equivocation".into()
    }

    fn string(&self) -> String {
        // TODO: sdk uses yaml
        todo!()
    }

    fn hash(&self) -> Hash {
        // TODO: check how we can guarantee infallible behavior
        Hash::from_bytes(Algorithm::Sha256, self.encode_to_vec().as_slice()).unwrap()
    }

    fn height(&self) -> i64 {
        self.height
    }

    fn handle<CTX: TransactionalContext<DB, SK>, DB: Database, SK: StoreKey>(
        _ctx: &mut CTX,
        _evidence: &Self,
    ) -> Result<(), <Self as Evidence>::Error> {
        // it is handled by module in the `begin_block`
        unreachable!()
    }
}
