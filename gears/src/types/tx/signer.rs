use core_types::address::AccAddress;
use serde::Deserialize;
use tendermint::types::chain_id::ChainId;

use crate::crypto::public::PublicKey;

/// SignerData is the specific information needed to sign a transaction that generally
/// isn't included in the transaction body itself
#[derive(Debug, Clone, Deserialize)]
pub struct SignerData {
    /// The address of the signer.
    ///
    /// In case of multisigs, this should be the multisig's address.
    pub address: AccAddress,

    /// ChainID is the chain that this transaction is targeting.
    pub chain_id: ChainId,

    /// AccountNumber is the account number of the signer.
    ///
    /// In case of multisigs, this should be the multisig account number.
    pub account_number: u64,

    /// Sequence is the account sequence number of the signer that is used
    /// for replay protection. This field is only useful for Legacy Amino signing,
    /// since in SIGN_MODE_DIRECT the account sequence is already in the signer info.
    ///
    /// In case of multisigs, this should be the multisig sequence.
    pub sequence: u64,

    pub pub_key: PublicKey,
}
