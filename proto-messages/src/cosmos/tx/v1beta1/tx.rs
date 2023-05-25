use std::str::FromStr;

use cosmwasm_std::Uint256;
use ibc_proto::{
    cosmos::base::v1beta1::Coin as RawCoin,
    cosmos::tx::v1beta1::{
        AuthInfo as RawAuthInfo, Fee as RawFee, ModeInfo, SignerInfo as RawSignerInfo,
        Tip as RawTip, Tx as RawTx, TxBody as RawTxBody,
    },
    google::protobuf::Any,
    protobuf::Protobuf,
};
use prost::bytes::Bytes;
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::{
    cosmos::base::v1beta1::{Coin, SendCoins},
    cosmos::{
        bank::v1beta1::MsgSend, base::abci::v1beta1::TxResponse,
        crypto::secp256k1::v1beta1::PubKey as Secp256k1PubKey,
    },
    error::Error,
};

pub const MAX_GAS_WANTED: u64 = 9223372036854775807; // = (1 << 63) -1 as specified in the cosmos SDK

/// Tx is the standard type used for broadcasting transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tx {
    /// body is the processable content of the transaction
    pub body: TxBody,
    /// auth_info is the authorization related content of the transaction,
    /// specifically signers, signer modes and fee
    pub auth_info: AuthInfo,
    /// signatures is a list of signatures that matches the length and order of
    /// AuthInfo's signer_infos to allow connecting signature meta information like
    /// public key and signing mode by position.
    #[serde(serialize_with = "crate::utils::serialize_vec_of_vec_to_vec_of_base64")]
    pub signatures: Vec<Vec<u8>>,
}

/// This enum is used where a Tx needs to be serialized like an Any
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum AnyTx {
    #[serde(rename = "/cosmos.tx.v1beta1.Tx")]
    Tx(Tx),
}

impl TryFrom<RawTx> for Tx {
    type Error = Error;

    fn try_from(raw: RawTx) -> Result<Self, Self::Error> {
        let body = raw.body.ok_or(Error::MissingField("body".into()))?;

        // This covers the SDK RejectExtensionOptions ante handler
        // https://github.com/cosmos/cosmos-sdk/blob/2582f0aab7b2cbf66ade066fe570a4622cf0b098/x/auth/ante/ext.go#L27-L36
        if !body.extension_options.is_empty() {
            return Err(Error::DecodeGeneral("unknown extension options".into()));
        }

        Ok(Tx {
            body: body.try_into()?,
            auth_info: raw
                .auth_info
                .ok_or(Error::MissingField("auth_info".into()))?
                .try_into()?,
            signatures: raw.signatures,
        })
    }
}

impl From<Tx> for RawTx {
    fn from(tx: Tx) -> RawTx {
        RawTx {
            body: Some(tx.body.into()),
            auth_info: Some(tx.auth_info.into()),
            signatures: tx.signatures,
        }
    }
}

impl Protobuf<RawTx> for Tx {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum Msg {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    Send(MsgSend),
}

impl Msg {
    pub fn get_signers(&self) -> Vec<&AccAddress> {
        match &self {
            Msg::Send(msg) => return vec![&msg.from_address],
        }
    }

    pub fn validate_basic(&self) -> Result<(), Error> {
        match &self {
            Msg::Send(_) => Ok(()),
        }
    }
}

impl From<Msg> for Any {
    fn from(msg: Msg) -> Self {
        match msg {
            Msg::Send(msg) => Any {
                type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                value: msg.encode_vec(),
            },
        }
    }
}

/// TxBody is the body of a transaction that all signers sign over.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TxBody {
    /// messages is a list of messages to be executed. The required signers of
    /// those messages define the number and order of elements in AuthInfo's
    /// signer_infos and Tx's signatures. Each required signer address is added to
    /// the list only the first time it occurs.
    /// By convention, the first required signer (usually from the first message)
    /// is referred to as the primary signer and pays the fee for the whole
    /// transaction.
    pub messages: Vec<Msg>,
    /// memo is any arbitrary note/comment to be added to the transaction.
    /// WARNING: in clients, any publicly exposed text should not be called memo,
    /// but should be called `note` instead (see <https://github.com/cosmos/cosmos-sdk/issues/9122>).
    pub memo: ::prost::alloc::string::String,
    /// timeout is the block height after which this transaction will not
    /// be processed by the chain
    #[serde_as(as = "DisplayFromStr")]
    pub timeout_height: u64,
    /// extension_options are arbitrary options that can be added by chains
    /// when the default options are not sufficient. If any of these are present
    /// and can't be handled, the transaction will be rejected
    pub extension_options: Vec<Any>, //TODO: use a domain type here
    /// extension_options are arbitrary options that can be added by chains
    /// when the default options are not sufficient. If any of these are present
    /// and can't be handled, they will be ignored
    pub non_critical_extension_options: Vec<Any>, //TODO: use a domain type here
}

impl TryFrom<RawTxBody> for TxBody {
    type Error = Error;

    fn try_from(raw: RawTxBody) -> Result<Self, Self::Error> {
        let mut messages: Vec<Msg> = vec![];

        for msg in &raw.messages {
            match msg.type_url.as_str() {
                "/cosmos.bank.v1beta1.MsgSend" => {
                    let msg = MsgSend::decode::<Bytes>(msg.value.clone().into())
                        .map_err(|e| Error::DecodeGeneral(e.to_string()))?;
                    messages.push(Msg::Send(msg));
                }
                _ => return Err(Error::DecodeGeneral("message type not recognized".into())), // If any message is not recognized then reject the entire Tx
            };
        }

        Ok(TxBody {
            messages,
            memo: raw.memo,
            timeout_height: raw.timeout_height,
            extension_options: raw.extension_options,
            non_critical_extension_options: raw.non_critical_extension_options,
        })
    }
}

impl From<TxBody> for RawTxBody {
    fn from(tx_body: TxBody) -> RawTxBody {
        RawTxBody {
            messages: tx_body.messages.into_iter().map(|m| m.into()).collect(),
            memo: tx_body.memo,
            timeout_height: tx_body.timeout_height,
            extension_options: tx_body.extension_options,
            non_critical_extension_options: tx_body.non_critical_extension_options,
        }
    }
}

impl Protobuf<RawTxBody> for TxBody {}

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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum PublicKey {
    #[serde(rename = "/cosmos.crypto.secp256k1.PubKey")]
    Secp256k1(Secp256k1PubKey),
    //Secp256r1(Vec<u8>),
    //Ed25519(Vec<u8>),
    //Multisig(Vec<u8>),
}

impl PublicKey {
    pub fn get_address(&self) -> AccAddress {
        match self {
            PublicKey::Secp256k1(key) => key.get_address(),
        }
    }
}

impl TryFrom<Any> for PublicKey {
    type Error = Error;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        match any.type_url.as_str() {
            "/cosmos.crypto.secp256k1.PubKey" => {
                let key = Secp256k1PubKey::decode::<Bytes>(any.value.into())
                    .map_err(|e| Error::DecodeGeneral(e.to_string()))?;
                Ok(PublicKey::Secp256k1(key))
            }
            _ => Err(Error::DecodeAny(format!(
                "Key type not recognized: {}",
                any.type_url
            ))),
        }
    }
}

impl From<PublicKey> for Any {
    fn from(key: PublicKey) -> Self {
        match key {
            PublicKey::Secp256k1(key) => Any {
                type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                value: key.encode_vec(),
            },
        }
    }
}

/// SignerInfo describes the public key and signing mode of a single top-level
/// signer.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignerInfo {
    /// public_key is the public key of the signer. It is optional for accounts
    /// that already exist in state. If unset, the verifier can use the required \
    /// signer address for this position and lookup the public key.
    pub public_key: Option<PublicKey>,
    /// mode_info describes the signing mode of the signer and is a nested
    /// structure to support nested multisig pubkey's
    pub mode_info: Option<ModeInfo>, // TODO: this isn't serializing correctly
    /// sequence is the sequence of the account, which describes the
    /// number of committed transactions signed by a given address. It is used to
    /// prevent replay attacks.
    #[serde_as(as = "DisplayFromStr")]
    pub sequence: u64,
}

impl TryFrom<RawSignerInfo> for SignerInfo {
    type Error = Error;

    fn try_from(raw: RawSignerInfo) -> Result<Self, Self::Error> {
        let key: Option<PublicKey> = match raw.public_key {
            Some(any) => Some(any.try_into()?),
            None => None,
        };
        Ok(SignerInfo {
            public_key: key,
            mode_info: raw.mode_info,
            sequence: raw.sequence,
        })
    }
}

impl From<SignerInfo> for RawSignerInfo {
    fn from(info: SignerInfo) -> RawSignerInfo {
        let key: Option<Any> = match info.public_key {
            Some(key) => Some(key.into()),
            None => None,
        };

        RawSignerInfo {
            public_key: key,
            mode_info: info.mode_info,
            sequence: info.sequence,
        }
    }
}

impl Protobuf<RawSignerInfo> for SignerInfo {}

/// Fee includes the amount of coins paid in fees and the maximum
/// gas to be used by the transaction. The ratio yields an effective "gasprice",
/// which must be above some miminum to be accepted into the mempool.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fee {
    /// amount is the amount of coins to be paid as a fee
    pub amount: Option<SendCoins>,
    /// gas_limit is the maximum gas that can be used in transaction processing
    /// before an out of gas error occurs
    #[serde_as(as = "DisplayFromStr")]
    pub gas_limit: u64,
    /// if unset, the first signer is responsible for paying the fees. If set, the specified account must pay the fees.
    /// the payer must be a tx signer (and thus have signed this field in AuthInfo).
    /// setting this field does *not* change the ordering of required signers for the transaction.
    pub payer: Option<AccAddress>,
    /// if set, the fee payer (either the first signer or the value of the payer field) requests that a fee grant be used
    /// to pay fees instead of the fee payer's own balance. If an appropriate fee grant does not exist or the chain does
    /// not support fee grants, this will fail
    pub granter: String,
}

impl TryFrom<RawFee> for Fee {
    type Error = Error;

    fn try_from(raw: RawFee) -> Result<Self, Self::Error> {
        if raw.gas_limit > MAX_GAS_WANTED {
            return Err(Error::DecodeGeneral(format!(
                "invalid gas supplied {} > {}",
                raw.gas_limit, MAX_GAS_WANTED
            )));
        }

        // There's a special case in the cosmos-sdk which allows the list of coins to be "invalid" provided
        // they're all zero - we'll check for this case and represent such a list of coins as a None fee amount.
        let mut all_zero = true;
        for coin in &raw.amount {
            let amount = Uint256::from_str(&coin.amount)
                .map_err(|_| Error::Coin(String::from("coin error")))?;
            if !amount.is_zero() {
                all_zero = false;
                break;
            }
        }

        let payer = match raw.payer.as_str() {
            "" => None,
            address => {
                let addr = AccAddress::from_bech32(address)
                    .map_err(|e| Error::DecodeAddress(e.to_string()))?;
                Some(addr)
            }
        };

        if all_zero {
            return Ok(Fee {
                amount: None,
                gas_limit: raw.gas_limit,
                payer,
                granter: raw.granter,
            });
        }

        let coins: Result<Vec<Coin>, Error> = raw
            .amount
            .into_iter()
            .map(|coin| Coin::try_from(coin))
            .collect();

        Ok(Fee {
            amount: Some(SendCoins::new(coins?)?),
            gas_limit: raw.gas_limit,
            payer: payer,
            granter: raw.granter,
        })
    }
}

impl From<Fee> for RawFee {
    fn from(fee: Fee) -> RawFee {
        let payer = match fee.payer {
            Some(addr) => addr.to_string(),
            None => "".into(),
        };
        match fee.amount {
            Some(amount) => {
                let coins: Vec<Coin> = amount.into();
                let coins = coins.into_iter().map(|coin| RawCoin::from(coin)).collect();

                RawFee {
                    amount: coins,
                    gas_limit: fee.gas_limit,
                    payer,
                    granter: fee.granter,
                }
            }
            None => RawFee {
                amount: vec![],
                gas_limit: fee.gas_limit,
                payer,
                granter: fee.granter,
            },
        }
    }
}

impl Protobuf<RawFee> for Fee {}

// Tip is the tip used for meta-transactions.
//
// Since: cosmos-sdk 0.46
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tip {
    /// amount is the amount of the tip
    pub amount: Option<SendCoins>,
    /// tipper is the address of the account paying for the tip
    pub tipper: AccAddress,
}

impl TryFrom<RawTip> for Tip {
    type Error = Error;

    fn try_from(raw: RawTip) -> Result<Self, Self::Error> {
        let tipper = AccAddress::from_bech32(&raw.tipper)
            .map_err(|e| Error::DecodeAddress(e.to_string()))?;

        let coins: Result<Vec<Coin>, Error> = raw
            .amount
            .into_iter()
            .map(|coin| Coin::try_from(coin))
            .collect();

        Ok(Tip {
            amount: Some(SendCoins::new(coins?)?),
            tipper,
        })
    }
}

impl From<Tip> for RawTip {
    fn from(tip: Tip) -> RawTip {
        let tipper = tip.tipper.to_string();

        match tip.amount {
            Some(amount) => {
                let coins: Vec<Coin> = amount.into();
                let coins = coins.into_iter().map(|coin| RawCoin::from(coin)).collect();

                RawTip {
                    amount: coins,
                    tipper,
                }
            }
            None => RawTip {
                amount: vec![],
                tipper,
            },
        }
    }
}

impl Protobuf<RawTip> for Tip {}

/// GetTxsEventResponse is the response type for the Service.TxsByEvents
/// RPC method.
#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTxsEventResponse {
    /// txs is the list of queried transactions.
    pub txs: Vec<Tx>,
    /// tx_responses is the list of queried TxResponses.
    pub tx_responses: Vec<TxResponse>,
    /// pagination defines a pagination for the response.
    /// Deprecated post v0.46.x: use total instead.
    // TODO: doesn't serialize correctly - has been deprecated
    pub pagination: Option<ibc_proto::cosmos::base::query::v1beta1::PageResponse>,
    /// total is total number of results available
    #[serde_as(as = "DisplayFromStr")]
    pub total: u64,
}

#[cfg(test)]
mod tests {

    use crate::cosmos::crypto::secp256k1::v1beta1::RawPubKey;

    use super::*;

    #[test]
    fn serialize_pubkey_works() {
        let key = hex::decode("02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c")
            .unwrap();
        let raw = RawPubKey { key };
        let key: Secp256k1PubKey = raw.try_into().unwrap();
        let key = PublicKey::Secp256k1(key);
        let key = serde_json::to_string(&key).unwrap();

        println!("{key}");

        assert_eq!(
            key,
            r#"{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"ApUOHN/LEz1gJBCf1In3NO60UCQY5TjChIHyK84nbySM"}"#
        );
    }

    #[test]
    fn deserialize_pubkey_works() {
        let serialized = r#"{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"ApUOHN/LEz1gJBCf1In3NO60UCQY5TjChIHyK84nbySM"}"#;
        let key: PublicKey = serde_json::from_str(serialized).unwrap();
        let PublicKey::Secp256k1(key) = key;
        assert_eq!(
            hex::encode(Vec::from(key)),
            "02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c"
        );
    }
}
