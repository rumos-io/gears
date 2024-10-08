mod handler;
pub mod param;
pub mod text;
pub mod upgrade;

use gears::error::ProtobufError;
pub use handler::*;
use ibc_proto::google::protobuf::Any;

pub trait ProposalModel:
    Clone
    + std::fmt::Debug
    + Send
    + Sync
    + 'static
    + serde::Serialize
    + TryFrom<Any, Error = ProtobufError>
    + Into<Any>
{
}
