use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid coins: `{0}`")]
    Coins(String),
    #[error("invalid coin: `{0}`")]
    Coin(String),
    #[error(transparent)]
    Decode(#[from] prost::DecodeError),
    #[error(transparent)]
    DecodeProtobuf(#[from] ibc_proto::protobuf::Error),
    #[error("decode error: `{0}`")]
    DecodeAny(String),
    #[error("missing field: `{0}`")]
    MissingField(String),
    #[error("decode error: `{0}`")]
    DecodeAddress(String),
    #[error("decode error: `{0}`")]
    DecodeGeneral(String),
}
