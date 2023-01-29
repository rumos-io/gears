pub mod v1beta1 {

    use ibc_proto::{
        cosmos::base::v1beta1::Coin as RawCoin,
        cosmos::tx::v1beta1::{
            AuthInfo as RawAuthInfo, Fee as RawFee, ModeInfo, SignerInfo as RawSignerInfo,
            Tx as RawTx, TxBody,
        },
        google::protobuf::Any,
        protobuf::Protobuf,
    };
    use prost::{bytes::Bytes, Message};
    use proto_types::AccAddress;

    use crate::{
        cosmos::base::v1beta1::{Coin, SendCoins},
        error::Error,
    };

    /// Tx is the standard type used for broadcasting transactions.
    #[derive(Clone, PartialEq)]
    pub struct Tx {
        /// body is the processable content of the transaction
        pub body: TxBody,
        /// auth_info is the authorization related content of the transaction,
        /// specifically signers, signer modes and fee
        pub auth_info: AuthInfo,
        /// signatures is a list of signatures that matches the length and order of
        /// AuthInfo's signer_infos to allow connecting signature meta information like
        /// public key and signing mode by position.
        pub signatures: Vec<Vec<u8>>,
    }

    impl TryFrom<RawTx> for Tx {
        type Error = Error;

        fn try_from(raw: RawTx) -> Result<Self, Self::Error> {
            Ok(Tx {
                body: raw.body.ok_or(Error::MissingField("body".into()))?,
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
                body: Some(tx.body),
                auth_info: Some(tx.auth_info.into()),
                signatures: tx.signatures,
            }
        }
    }

    impl Protobuf<RawTx> for Tx {}

    /// AuthInfo describes the fee and signer modes that are used to sign a
    /// transaction.
    #[derive(Clone, PartialEq)]
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
    }

    impl TryFrom<RawAuthInfo> for AuthInfo {
        type Error = Error;

        fn try_from(raw: RawAuthInfo) -> Result<Self, Self::Error> {
            let signer_infos: Result<Vec<SignerInfo>, Error> = raw
                .signer_infos
                .into_iter()
                .map(|info| info.try_into())
                .collect();

            Ok(AuthInfo {
                signer_infos: signer_infos?,
                fee: raw
                    .fee
                    .ok_or(Error::MissingField(String::from("fee")))?
                    .try_into()?,
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
            }
        }
    }

    impl Protobuf<RawAuthInfo> for AuthInfo {}

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Secp256k1 {
        #[prost(bytes = "vec", tag = "1")]
        pub key: Vec<u8>,
    }

    #[derive(Clone, PartialEq)]
    pub enum PublicKey {
        Secp256k1(Secp256k1),
        //Secp256r1(Vec<u8>),
        //Ed25519(Vec<u8>),
        //Multisig(Vec<u8>),
    }

    impl TryFrom<Any> for PublicKey {
        type Error = Error;

        fn try_from(any: Any) -> Result<Self, Self::Error> {
            match any.type_url.as_str() {
                "/cosmos.crypto.secp256k1.PubKey" => {
                    let key = Secp256k1::decode::<Bytes>(any.value.into())?;
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
                    value: key.encode_to_vec(),
                },
            }
        }
    }

    /// SignerInfo describes the public key and signing mode of a single top-level
    /// signer.
    #[derive(Clone, PartialEq)]
    pub struct SignerInfo {
        /// public_key is the public key of the signer. It is optional for accounts
        /// that already exist in state. If unset, the verifier can use the required \
        /// signer address for this position and lookup the public key.
        pub public_key: Option<PublicKey>,
        /// mode_info describes the signing mode of the signer and is a nested
        /// structure to support nested multisig pubkey's
        pub mode_info: Option<ModeInfo>,
        /// sequence is the sequence of the account, which describes the
        /// number of committed transactions signed by a given address. It is used to
        /// prevent replay attacks.
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
    #[derive(Clone, PartialEq)]
    pub struct Fee {
        /// amount is the amount of coins to be paid as a fee
        pub amount: Option<SendCoins>,
        /// gas_limit is the maximum gas that can be used in transaction processing
        /// before an out of gas error occurs
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
            // There's a special case in the cosmos-sdk which allows the list of coins to be "invalid" provided
            // they're all zero - we'll check for this case and represent such a list of coins as a None fee amount.
            let mut all_zero = true;
            for coin in &raw.amount {
                if !coin.amount.is_zero() {
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
}
