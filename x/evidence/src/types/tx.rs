use gears::{
    core::any::google::Any,
    derive::{AppMessage, Protobuf, Raw},
    types::address::AccAddress,
};
use serde::{Deserialize, Serialize};

/// MsgSubmitEvidence represents a message that supports submitting arbitrary
/// Evidence of misbehavior such as equivocation or counterfactual signing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Raw, Protobuf, AppMessage)]
#[msg(url = "/cosmos.evidence.v1beta1.SubmitEvidence")]
pub struct MsgSubmitEvidence {
    #[raw(kind(string), raw = String)]
    #[msg(signer)]
    pub submitter: AccAddress,
    #[raw(kind(message), raw = Any)]
    pub evidence: Any,
}
