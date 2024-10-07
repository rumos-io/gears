use std::num::NonZero;

use gears::{
    context::QueryableContext,
    core::{errors::CoreError, Protobuf},
    error::ProtobufError,
};

mod inner {
    pub use ibc_proto::cosmos::upgrade::v1beta1::Plan;
}

#[nutype::nutype(
    validate(not_empty),
    derive(Debug, Clone, Serialize, Deserialize, AsRef)
)]
pub struct PlanName(String);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Plan {
    pub name: PlanName,
    pub height: NonZero<u32>,
    pub info: String,
}

impl Plan {
    pub fn should_execute<CTX: QueryableContext<DB, SK>, DB, SK>(&self, ctx: &CTX) -> bool {
        self.height.get() <= ctx.height()
    }
}

impl From<Plan> for inner::Plan {
    fn from(Plan { name, height, info }: Plan) -> Self {
        #[allow(deprecated)]
        Self {
            name: name.into_inner(),
            time: None,
            height: height.get().into(),
            info,
            upgraded_client_state: None,
        }
    }
}

impl TryFrom<inner::Plan> for Plan {
    type Error = ProtobufError;

    #[allow(deprecated)]
    fn try_from(
        inner::Plan {
            name,
            time,
            height,
            info,
            upgraded_client_state,
        }: inner::Plan,
    ) -> Result<Self, Self::Error> {
        if time.is_some() || upgraded_client_state.is_some() {
            Err(anyhow::anyhow!(
                "`time` and `upgraded_client_state` is deprecated"
            ))?
        }

        Ok(Self {
            name: PlanName::try_new(name)
                .map_err(|_| ProtobufError::MissingField("`name` is empty".to_owned()))?,
            height: u32::try_from(height)
                .map_err(|e| e.to_string())
                .and_then(|this| NonZero::new(this).ok_or("height can't be zero".to_owned()))
                .map_err(|e| CoreError::DecodeGeneral(format!("invalid `height`: {e}")))?,
            info,
        })
    }
}

impl Protobuf<inner::Plan> for Plan {}
