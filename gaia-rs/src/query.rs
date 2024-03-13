use auth::cli::query::{AuthQuery, AuthQueryResponse, RawAuthQueryResponse};
use bank::cli::query::{BankQuery, BankQueryResponse, RawBankQueryResponse};
use ibc::cli::client::query::{IbcProtoError, IbcQuery, IbcQueryResponse, RawIbcQueryResponse};
use prost::encoding::{DecodeContext, WireType};
use proto_messages::cosmos::{ibc::protobuf::Protobuf, query::Query};
use serde::Serialize;

#[derive(Clone, PartialEq)]
pub enum GaiaQuery {
    Auth(AuthQuery),
    Bank(BankQuery),
    Ibc(IbcQuery),
}

impl Query for GaiaQuery {
    fn query_url(&self) -> std::borrow::Cow<'static, str> {
        match self {
            GaiaQuery::Auth(var) => var.query_url(),
            GaiaQuery::Bank(var) => var.query_url(),
            GaiaQuery::Ibc(var) => var.query_url(),
        }
    }

    fn as_bytes(self) -> Vec<u8> {
        match self {
            GaiaQuery::Auth(var) => var.as_bytes(),
            GaiaQuery::Bank(var) => var.as_bytes(),
            GaiaQuery::Ibc(var) => var.as_bytes(),
        }
    }
}

#[derive(Clone, Serialize)]
pub enum GaiaQueryResponse {
    Auth(AuthQueryResponse),
    Bank(BankQueryResponse),
    Ibc(IbcQueryResponse),
}

#[derive(Debug, Clone)]
pub enum RawGaiaQueryResponse {
    Auth(RawAuthQueryResponse),
    Bank(RawBankQueryResponse),
    Ibc(RawIbcQueryResponse),
}

impl Default for RawGaiaQueryResponse {
    fn default() -> Self {
        Self::Auth(Default::default())
    }
}

impl Protobuf<RawGaiaQueryResponse> for GaiaQueryResponse {}

impl From<GaiaQueryResponse> for RawGaiaQueryResponse {
    fn from(value: GaiaQueryResponse) -> Self {
        match value {
            GaiaQueryResponse::Auth(var) => Self::Auth(var.into()),
            GaiaQueryResponse::Bank(var) => Self::Bank(var.into()),
            GaiaQueryResponse::Ibc(var) => Self::Ibc(var.into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GaiaProtoError {
    #[error("{0}")]
    Ibc(#[from] IbcProtoError),
    #[error("{0}")]
    Proto(#[from] proto_messages::Error),
}

impl TryFrom<RawGaiaQueryResponse> for GaiaQueryResponse {
    type Error = GaiaProtoError;

    fn try_from(value: RawGaiaQueryResponse) -> Result<Self, Self::Error> {
        let res = match value {
            RawGaiaQueryResponse::Auth(var) => Self::Auth(var.try_into()?),
            RawGaiaQueryResponse::Bank(var) => Self::Bank(var.try_into()?),
            RawGaiaQueryResponse::Ibc(var) => Self::Ibc(var.try_into()?),
        };

        Ok(res)
    }
}

impl prost::Message for RawGaiaQueryResponse {
    fn encode_raw<B>(&self, buf: &mut B)
    where
        B: bytes::BufMut,
        Self: Sized,
    {
        match self {
            RawGaiaQueryResponse::Auth(var) => var.encode_raw(buf),
            RawGaiaQueryResponse::Bank(var) => var.encode_raw(buf),
            RawGaiaQueryResponse::Ibc(var) => var.encode_raw(buf),
        }
    }

    fn merge_field<B>(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut B,
        ctx: DecodeContext,
    ) -> Result<(), prost::DecodeError>
    where
        B: bytes::Buf,
        Self: Sized,
    {
        match self {
            RawGaiaQueryResponse::Auth(var) => var.merge_field(tag, wire_type, buf, ctx),
            RawGaiaQueryResponse::Bank(var) => var.merge_field(tag, wire_type, buf, ctx),
            RawGaiaQueryResponse::Ibc(var) => var.merge_field(tag, wire_type, buf, ctx),
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            RawGaiaQueryResponse::Auth(var) => var.encoded_len(),
            RawGaiaQueryResponse::Bank(var) => var.encoded_len(),
            RawGaiaQueryResponse::Ibc(var) => var.encoded_len(),
        }
    }

    fn clear(&mut self) {
        match self {
            RawGaiaQueryResponse::Auth(var) => var.clear(),
            RawGaiaQueryResponse::Bank(var) => var.clear(),
            RawGaiaQueryResponse::Ibc(var) => var.clear(),
        }
    }
}
