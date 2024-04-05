use crate::{address::AccAddress, any::Any};

use self::error::TxError;

pub mod error;

pub trait TxMessage:
    serde::Serialize + Clone + Send + Sync + 'static + Into<Any> + TryFrom<Any, Error = TxError>
{
    fn get_signers(&self) -> Vec<&AccAddress>;

    fn validate_basic(&self) -> Result<(), String>;

    fn type_url(&self) -> &'static str; // TODO:NOW Cow<'static, str>?
}
