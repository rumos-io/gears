use std::{collections::HashSet, str::FromStr};

use cosmwasm_std::Uint256;
use ibc_proto::{
    cosmos::base::v1beta1::Coin as RawCoin,
    cosmos::tx::v1beta1::{
        AuthInfo as RawAuthInfo, Fee as RawFee, ModeInfo, SignerInfo as RawSignerInfo,
        Tip as RawTip, Tx as RawTx, TxBody as RawTxBody, TxRaw,
    },
    google::protobuf::Any,
    protobuf::Protobuf,
};
use prost::{bytes::Bytes, Message as ProstMessage};
use proto_types::AccAddress;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::{
    cosmos::base::v1beta1::{Coin, SendCoins},
    cosmos::{
        base::abci::v1beta1::TxResponse, crypto::secp256k1::v1beta1::PubKey as Secp256k1PubKey,
    },
    error::Error,
};

pub const MAX_GAS_WANTED: u64 = 9223372036854775807; // = (1 << 63) -1 as specified in the cosmos SDK