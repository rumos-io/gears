use ibc_proto::{
    cosmos::base::v1beta1::Coin as RawCoin, cosmos::tx::v1beta1::SignerInfo as SignerInfoRaw,
    google::protobuf::Any, protobuf::Protobuf,
};
use prost::{bytes::Bytes, Message as ProstMessage};

use crate::{
    cosmos::{
        base::v1beta1::Coin,
        crypto::secp256k1::v1beta1::{PubKey, RawPubKey},
    },
    Error,
};

use super::{signer::SignerInfo, signer_data::ChainId};

#[derive(Clone, PartialEq, ProstMessage)]
pub struct EnvelopeRaw {
    #[prost(string, tag = "1")]
    pub chain_id: String, // `protobuf:"bytes,1,opt,name=chain_id,json=chainId,proto3" json:"chain_id,omitempty"`
    #[prost(fixed64)]
    pub account_number: u64, //`protobuf:"varint,2,opt,name=account_number,json=accountNumber,proto3" json:"account_number,omitempty"`
    #[prost(fixed64)]
    pub sequence: u64, // `protobuf:"varint,3,opt,name=sequence,proto3" json:"sequence,omitempty"`
    #[prost(string)]
    pub address: String, // `protobuf:"bytes,4,opt,name=address,proto3" json:"address,omitempty"`
    #[prost(message, required)]
    pub public_key: RawPubKey, // `protobuf:"bytes,5,opt,name=public_key,json=publicKey,proto3" json:"public_key,omitempty"`
    #[prost(message, repeated)]
    pub message: Vec<Any>, //  `protobuf:"bytes,6,rep,name=message,proto3" json:"message,omitempty"`
    #[prost(string)]
    pub memo: String, // `protobuf:"bytes,7,opt,name=memo,proto3" json:"memo,omitempty"`
    #[prost(message, repeated)]
    pub fees: Vec<RawCoin>, // `protobuf:"bytes,8,rep,name=fees,proto3" json:"fees,omitempty"`
    #[prost(bytes)]
    pub fee_payer: Bytes, //  `protobuf:"bytes,9,opt,name=fee_payer,json=feePayer,proto3" json:"fee_payer,omitempty"`
    #[prost(string)]
    pub fee_granter: String, //  `protobuf:"bytes,10,opt,name=fee_granter,json=feeGranter,proto3" json:"fee_granter,omitempty"`
    #[prost(message, repeated)]
    pub tip: Vec<RawCoin>, // `protobuf:"bytes,11,rep,name=tip,proto3" json:"tip,omitempty"`
    #[prost(string)]
    pub tipper: String, // `protobuf:"bytes,12,opt,name=tipper,proto3" json:"tipper,omitempty"`
    #[prost(fixed64)]
    pub gas_limit: u64, // `protobuf:"varint,13,opt,name=gas_limit,json=gasLimit,proto3" json:"gas_limit,omitempty"`
    #[prost(fixed64)]
    pub timeout_height: u64, // `protobuf:"varint,14,opt,name=timeout_height,json=timeoutHeight,proto3" json:"timeout_height,omitempty"`
    #[prost(message, repeated)]
    pub other_signer: Vec<SignerInfoRaw>, // `protobuf:"bytes,15,rep,name=other_signer,json=otherSigner,proto3" json:"other_signer,omitempty"`
    #[prost(message, repeated)]
    pub extensions_options: Vec<Any>, //  `protobuf:"bytes,16,rep,name=extension_options,json=extensionOptions,proto3" json:"extension_options,omitempty"`
    #[prost(message, repeated)]
    pub non_critical_ext_opt: Vec<Any>, // `protobuf:"bytes,17,rep,name=non_critical_extension_options,json=nonCriticalExtensionOptions,proto3" json:"non_critical_extension_options,omitempty"`
    #[prost(string)]
    pub hash_of_raw_bytes: String, // `protobuf:"bytes,18,opt,name=hash_of_raw_bytes,json=hashOfRawBytes,proto3" json:"hash_of_raw_bytes,omitempty"`
}

#[derive(Debug, Clone)]
pub struct Envelope {
    pub chain_id: ChainId,
    pub account_number: u64,
    pub sequence: u64,
    pub address: String,
    pub public_key: PubKey,
    pub memo: String,
    pub fees: Vec<Coin>,
    pub fee_payer: Bytes,
    pub fee_granter: String,
    pub tip: Vec<Coin>,
    pub tipper: String,
    pub gas_limit: u64,
    pub timeout_height: u64,
    pub other_signer: Vec<SignerInfo>,
    pub hash_of_raw_bytes: String,
}

impl Protobuf<EnvelopeRaw> for Envelope {}

impl TryFrom<EnvelopeRaw> for Envelope {
    type Error = Error;

    fn try_from(value: EnvelopeRaw) -> Result<Self, Self::Error> {
        let EnvelopeRaw {
            chain_id,
            account_number,
            sequence,
            address,
            public_key,
            memo,
            fees,
            fee_payer,
            fee_granter,
            tip,
            tipper,
            gas_limit,
            timeout_height,
            other_signer,
            hash_of_raw_bytes,
            .. // other field currently ignored and immediatly dropped
        } = value;

        let mut mapped_signers = Vec::with_capacity(other_signer.len());
        for var in other_signer {
            mapped_signers.push(var.try_into()?);
        }

        let mut mapped_fees = Vec::with_capacity(fees.len());
        for var in fees {
            mapped_fees.push(var.try_into()?);
        }

        let mut mapped_tips = Vec::with_capacity(tip.len());
        for var in tip {
            mapped_tips.push(var.try_into()?);
        }

        let var = Self {
            chain_id: ChainId::new(chain_id).map_err(|e| Error::DecodeGeneral(e.to_string()))?,
            account_number,
            sequence,
            address,
            public_key: public_key.try_into()?,
            memo,
            fees: mapped_fees,
            fee_payer,
            fee_granter,
            tip: mapped_tips,
            tipper,
            gas_limit,
            timeout_height,
            other_signer: mapped_signers,
            hash_of_raw_bytes,
        };

        Ok(var)
    }
}

impl From<Envelope> for EnvelopeRaw {
    fn from(value: Envelope) -> Self {
        let Envelope {
            chain_id,
            account_number,
            sequence,
            address,
            public_key,
            memo,
            fees,
            fee_payer,
            fee_granter,
            tip,
            tipper,
            gas_limit,
            timeout_height,
            other_signer,
            hash_of_raw_bytes,
        } = value;

        Self {
            chain_id: chain_id.into_inner(),
            account_number,
            sequence,
            address,
            public_key: public_key.into(),
            message: Vec::new(),
            memo,
            fees: fees.into_iter().map(|this| this.into()).collect(),
            fee_payer,
            fee_granter,
            tip: tip.into_iter().map(|this| this.into()).collect(),
            tipper,
            gas_limit,
            timeout_height,
            other_signer: other_signer.into_iter().map(|this| this.into()).collect(),
            extensions_options: Vec::new(),
            non_critical_ext_opt: Vec::new(),
            hash_of_raw_bytes,
        }
    }
}
