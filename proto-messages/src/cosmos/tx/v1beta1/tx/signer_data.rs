use nutype::nutype;

use crate::cosmos::crypto::secp256k1::v1beta1::PubKey;

#[nutype(validate(not_empty))]
#[derive(*)]
pub struct ChainId(String);

/// SignerData is the specific information needed to sign a transaction that generally
/// isn't included in the transaction body itself
#[derive(Debug)]
pub struct SignerData {
    /// The address of the signer.
    ///
    /// In case of multisigs, this should be the multisig's address.
    pub address: String,

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

    pub pub_key: PubKey,
}
