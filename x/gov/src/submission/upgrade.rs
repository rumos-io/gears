use gears::{derive::Protobuf, types::address::AccAddress};
use serde::{Deserialize, Serialize};
use upgrade::types::plan::Plan;

mod inner {
    pub use ibc_proto::cosmos::upgrade::v1beta1::MsgCancelUpgrade;
    pub use ibc_proto::cosmos::upgrade::v1beta1::MsgSoftwareUpgrade;
}

#[derive(Debug, Clone, Protobuf, Serialize, Deserialize)]
#[proto(raw = "inner::MsgCancelUpgrade")]
pub struct MsgCancelUpgrade {
    pub authority: AccAddress,
}

#[derive(Debug, Clone, Protobuf, Serialize, Deserialize)]
#[proto(raw = "inner::MsgSoftwareUpgrade")]
pub struct MsgSoftwareUpgrade {
    pub authority: AccAddress,
    #[proto(optional)]
    pub plan: Plan,
}
