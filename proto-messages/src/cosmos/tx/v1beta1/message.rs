use ibc_proto::google::protobuf::Any;
use proto_types::AccAddress;
use serde::Serialize;

use crate::error::Error;

pub trait Message:
    Serialize + Clone + Send + Sync + 'static + Into<Any> + TryFrom<Any, Error = Error>
{
    //fn decode(raw: &Any) -> Self; // TODO: could be From<Any>

    fn get_signers(&self) -> Vec<&AccAddress>;

    fn validate_basic(&self) -> Result<(), String>;
}
