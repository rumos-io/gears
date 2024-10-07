use gears::{
    context::QueryableContext,
    core::{errors::CoreError, Protobuf},
    error::ProtobufError,
};

mod inner {
    pub use ibc_proto::cosmos::upgrade::v1beta1::Plan;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Plan {
    pub name: String,
    pub height: u32,
    pub info: String,
}

impl Plan {
    pub fn should_execute<CTX: QueryableContext<DB, SK>, DB, SK>(&self, ctx: &CTX) -> bool {
        match self.height > 0 {
            true => self.height <= ctx.height(),
            false => false,
        }
    }
}

impl From<Plan> for inner::Plan {
    fn from(Plan { name, height, info }: Plan) -> Self {
        #[allow(deprecated)]
        Self {
            name,
            time: None,
            height: height.into(),
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
            name,
            height: height
                .try_into()
                .map_err(|e| CoreError::DecodeGeneral(format!("invalid `height`: {e}")))?,
            info,
        })
    }
}

impl Protobuf<inner::Plan> for Plan {}
