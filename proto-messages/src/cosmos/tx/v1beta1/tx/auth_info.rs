use ibc_proto::protobuf::Protobuf;

use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::{fee::Fee, signer::SignerInfo, tip::Tip};

pub use ibc_proto::cosmos::tx::v1beta1::AuthInfo as RawAuthInfo;

/// AuthInfo describes the fee and signer modes that are used to sign a
/// transaction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthInfo {
    /// signer_infos defines the signing modes for the required signers. The number
    /// and order of elements must match the required signers from TxBody's
    /// messages. The first element is the primary signer and the one which pays
    /// the fee.
    pub signer_infos: Vec<SignerInfo>,
    /// Fee is the fee and gas limit for the transaction. The first signer is the
    /// primary signer and the one which pays the fee. The fee can be calculated
    /// based on the cost of evaluating the body and doing signature verification
    /// of the signers. This can be estimated via simulation.
    pub fee: Fee,
    // Tip is the optional tip used for transactions fees paid in another denom.
    //
    // This field is ignored if the chain didn't enable tips, i.e. didn't add the
    // `TipDecorator` in its posthandler.
    //
    // Since: cosmos-sdk 0.46
    pub tip: Option<Tip>,
}

impl TryFrom<RawAuthInfo> for AuthInfo {
    type Error = Error;

    fn try_from(raw: RawAuthInfo) -> Result<Self, Self::Error> {
        let signer_infos: Result<Vec<SignerInfo>, Error> = raw
            .signer_infos
            .into_iter()
            .map(|info| info.try_into())
            .collect();

        let tip = raw.tip.map(|tip| tip.try_into()).transpose()?;

        Ok(AuthInfo {
            signer_infos: signer_infos?,
            fee: raw
                .fee
                .ok_or(Error::MissingField(String::from("fee")))?
                .try_into()?,
            tip,
        })
    }
}

impl From<AuthInfo> for RawAuthInfo {
    fn from(auth_info: AuthInfo) -> RawAuthInfo {
        let sig_infos: Vec<SignerInfo> = auth_info.signer_infos;
        let sig_infos = sig_infos
            .into_iter()
            .map(|sig_info| sig_info.into())
            .collect();

        RawAuthInfo {
            signer_infos: sig_infos,
            fee: Some(auth_info.fee.into()),
            tip: auth_info.tip.map(|tip| tip.into()),
        }
    }
}

impl Protobuf<RawAuthInfo> for AuthInfo {}
