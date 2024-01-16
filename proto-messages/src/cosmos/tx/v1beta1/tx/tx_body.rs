use ibc_proto::{
    cosmos::tx::v1beta1::TxBody as RawTxBody, google::protobuf::Any, protobuf::Protobuf,
};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::{cosmos::tx::v1beta1::message::Message, error::Error};

// TxBody is the body of a transaction that all signers sign over.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TxBody<M: Message> {
    /// messages is a list of messages to be executed. The required signers of
    /// those messages define the number and order of elements in AuthInfo's
    /// signer_infos and Tx's signatures. Each required signer address is added to
    /// the list only the first time it occurs.
    /// By convention, the first required signer (usually from the first message)
    /// is referred to as the primary signer and pays the fee for the whole
    /// transaction.
    pub messages: Vec<M>,
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

impl<M: Message> TryFrom<RawTxBody> for TxBody<M> {
    type Error = Error;

    fn try_from(raw: RawTxBody) -> Result<Self, Self::Error> {
        let mut messages: Vec<M> = vec![];

        for msg in &raw.messages {
            messages.push(Any::try_into(msg.to_owned())?);
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

impl<M: Message> From<TxBody<M>> for RawTxBody {
    fn from(tx_body: TxBody<M>) -> RawTxBody {
        RawTxBody {
            messages: tx_body.messages.into_iter().map(|m| m.into()).collect(),
            memo: tx_body.memo,
            timeout_height: tx_body.timeout_height,
            extension_options: tx_body.extension_options,
            non_critical_extension_options: tx_body.non_critical_extension_options,
        }
    }
}

impl<M: Message> Protobuf<RawTxBody> for TxBody<M> {}
