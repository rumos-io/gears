use ibc_proto::cosmos::tx::v1beta1::{
    mode_info::{Multi as RawMulti, Single as RawSingle, Sum as RawSum},
    ModeInfo as RawModeInfo,
};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModeInfo {
    /// single represents a single signer
    Single(SignMode),
    /// multi represents a nested multisig signer
    Multi(Multi),
}

impl TryFrom<RawModeInfo> for ModeInfo {
    type Error = Error;

    fn try_from(raw: RawModeInfo) -> Result<Self, Self::Error> {
        Ok(
            match raw.sum.ok_or(Error::MissingField(String::from("sum")))? {
                RawSum::Single(s) => Self::Single(s.try_into()?),
                RawSum::Multi(m) => Self::Multi(m.try_into()?),
            },
        )
    }
}

impl From<ModeInfo> for RawModeInfo {
    fn from(mode_info: ModeInfo) -> RawModeInfo {
        match mode_info {
            ModeInfo::Single(s) => RawModeInfo {
                sum: Some(RawSum::Single(s.into())),
            },
            ModeInfo::Multi(m) => RawModeInfo {
                sum: Some(RawSum::Multi(m.into())),
            },
        }
    }
}

// https://github.com/joneskm/ibc-proto-rs/blob/935941cedfd3d1cf87abbc3505d4cdcbc74b15e9/src/prost/cosmos.tx.signing.v1beta1.rs#L95C1-L127C2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SignMode {
    /// SIGN_MODE_UNSPECIFIED specifies an unknown signing mode and will be
    /// rejected.
    Unspecified = 0,
    /// SIGN_MODE_DIRECT specifies a signing mode which uses SignDoc and is
    /// verified with raw bytes from Tx.
    Direct = 1,
    /// SIGN_MODE_TEXTUAL is a future signing mode that will verify some
    /// human-readable textual representation on top of the binary representation
    /// from SIGN_MODE_DIRECT. It is currently not supported.
    Textual = 2,
    /// SIGN_MODE_DIRECT_AUX specifies a signing mode which uses
    /// SignDocDirectAux. As opposed to SIGN_MODE_DIRECT, this sign mode does not
    /// require signers signing over other signers' `signer_info`. It also allows
    /// for adding Tips in transactions.
    ///
    /// Since: cosmos-sdk 0.46
    DirectAux = 3,
    /// SIGN_MODE_LEGACY_AMINO_JSON is a backwards compatibility mode which uses
    /// Amino JSON and will be removed in the future.
    LegacyAminoJson = 127,
    /// SIGN_MODE_EIP_191 specifies the sign mode for EIP 191 signing on the Cosmos
    /// SDK. Ref: <https://eips.ethereum.org/EIPS/eip-191>
    ///
    /// Currently, SIGN_MODE_EIP_191 is registered as a SignMode enum variant,
    /// but is not implemented on the SDK by default. To enable EIP-191, you need
    /// to pass a custom `TxConfig` that has an implementation of
    /// `SignModeHandler` for EIP-191. The SDK may decide to fully support
    /// EIP-191 in the future.
    ///
    /// Since: cosmos-sdk 0.45.2
    Eip191 = 191,
}

impl From<SignMode> for RawSingle {
    fn from(value: SignMode) -> Self {
        RawSingle { mode: value as i32 }
    }
}

impl TryFrom<RawSingle> for SignMode {
    type Error = Error;

    fn try_from(raw: RawSingle) -> Result<Self, Self::Error> {
        Ok(match raw.mode {
            0 => SignMode::Unspecified,
            1 => SignMode::Direct,
            2 => SignMode::Textual,
            3 => SignMode::DirectAux,
            127 => SignMode::LegacyAminoJson,
            191 => SignMode::Eip191,
            _ => return Err(Error::InvalidSignMode(raw.mode)),
        })
    }
}

/// Multi is the mode info for a multisig public key
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Multi {
    /// bitarray specifies which keys within the multisig are signing
    pub bitarray: Option<ibc_proto::cosmos::crypto::multisig::v1beta1::CompactBitArray>,
    /// mode_infos is the corresponding modes of the signers of the multisig
    /// which could include nested multisig public keys
    pub mode_infos: Vec<ModeInfo>,
}

impl TryFrom<RawMulti> for Multi {
    type Error = Error;

    fn try_from(raw: RawMulti) -> Result<Self, Self::Error> {
        Ok(Multi {
            bitarray: raw.bitarray,
            mode_infos: raw
                .mode_infos
                .into_iter()
                .map(|mi| mi.try_into())
                .collect::<Result<Vec<ModeInfo>, Error>>()?,
        })
    }
}

impl From<Multi> for RawMulti {
    fn from(multi: Multi) -> RawMulti {
        RawMulti {
            bitarray: multi.bitarray,
            mode_infos: multi
                .mode_infos
                .into_iter()
                .map(RawModeInfo::from)
                .collect(),
        }
    }
}
