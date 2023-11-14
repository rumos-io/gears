use std::marker::PhantomData;

use database::Database;
use ibc_proto::cosmos::tx::v1beta1::SignDoc;

use prost::Message as ProstMessage;
use proto_messages::cosmos::{
    auth::v1beta1::Account,
    base::v1beta1::SendCoins,
    tx::v1beta1::{message::Message, public_key::PublicKey, tx::tx::Tx, tx_raw::TxWithRaw},
};
use proto_types::AccAddress;
use secp256k1::{ecdsa, hashes::sha256, PublicKey as Secp256k1PubKey, Secp256k1};
use store_crate::StoreKey;

use crate::types::context::context::Context;
use crate::{
    error::AppError,
    x::auth::{Module, Params},
};

// TODO: this doesn't belong here
pub trait BankKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    fn send_coins_from_account_to_module<DB: Database>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        from_address: AccAddress,
        to_module: Module,
        amount: SendCoins,
    ) -> Result<(), AppError>;
}

// TODO: this doesn't belong here
pub trait AuthKeeper<SK: StoreKey>: Clone + Send + Sync + 'static {
    fn get_auth_params<DB: Database>(&self, ctx: &Context<'_, '_, DB, SK>) -> Params;

    fn has_account<DB: Database>(&self, ctx: &Context<'_, '_, DB, SK>, addr: &AccAddress) -> bool;

    fn get_account<DB: Database>(
        &self,
        ctx: &Context<'_, '_, DB, SK>,
        addr: &AccAddress,
    ) -> Option<Account>;

    fn set_account<DB: Database>(&self, ctx: &mut Context<'_, '_, DB, SK>, acct: Account);
}

#[derive(Debug, Clone)]
pub struct AnteHandler<BK: BankKeeper<SK>, AK: AuthKeeper<SK>, SK: StoreKey> {
    bank_keeper: BK,
    auth_keeper: AK,
    sk: PhantomData<SK>,
}

impl<BK: BankKeeper<SK>, AK: AuthKeeper<SK>, SK: StoreKey> AnteHandler<BK, AK, SK> {
    pub fn new(bank_keeper: BK, auth_keeper: AK) -> AnteHandler<BK, AK, SK> {
        AnteHandler {
            bank_keeper,
            auth_keeper,
            sk: PhantomData,
        }
    }
    pub fn run<DB: Database, M: Message>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError> {
        self.validate_basic_ante_handler(&tx.tx)?;
        self.tx_timeout_height_ante_handler(ctx, &tx.tx)?;
        self.validate_memo_ante_handler(ctx, &tx.tx)?;
        self.deduct_fee_ante_handler(ctx, &tx.tx)?;
        self.set_pub_key_ante_handler(ctx, &tx.tx)?;
        self.sig_verification_handler(ctx, tx)?;
        self.increment_sequence_ante_handler(ctx, &tx.tx)?;

        //  ** ante.NewSetUpContextDecorator(),
        //  - ante.NewRejectExtensionOptionsDecorator(), // Covered in tx parsing code
        //  - NewMempoolFeeDecorator(opts.BypassMinFeeMsgTypes), // NOT USED FOR DELIVER_TX
        //  - ante.NewValidateBasicDecorator(),
        //  - ante.NewTxTimeoutHeightDecorator(),
        //  - ante.NewValidateMemoDecorator(opts.AccountKeeper),
        //  ** ante.NewConsumeGasForTxSizeDecorator(opts.AccountKeeper),
        //  - ante.NewDeductFeeDecorator(opts.AccountKeeper, opts.BankKeeper, opts.FeegrantKeeper),
        // // SetPubKeyDecorator must be called before all signature verification decorators
        //  - ante.NewSetPubKeyDecorator(opts.AccountKeeper),
        //  ** ante.NewValidateSigCountDecorator(opts.AccountKeeper),
        //  ** ante.NewSigGasConsumeDecorator(opts.AccountKeeper, sigGasConsumer),
        //  - ante.NewSigVerificationDecorator(opts.AccountKeeper, opts.SignModeHandler),
        //  - ante.NewIncrementSequenceDecorator(opts.AccountKeeper),
        //  ** ibcante.NewAnteDecorator(opts.IBCkeeper),

        Ok(())
    }

    fn validate_basic_ante_handler<M: Message>(&self, tx: &Tx<M>) -> Result<(), AppError> {
        // Not sure if we need to explicitly check this given the check which follows.
        // We'll leave it in for now since it's in the SDK.
        let sigs = tx.get_signatures();
        if sigs.is_empty() {
            return Err(AppError::TxValidation("signature list is empty".into()));
        }

        if sigs.len() != tx.get_signers().len() {
            return Err(AppError::TxValidation(format!(
                "wrong number of signatures; expected {}, got {}",
                tx.get_signers().len(),
                sigs.len()
            )));
        }

        Ok(())
    }

    fn tx_timeout_height_ante_handler<DB: Database, M: Message>(
        &self,
        ctx: &Context<'_, '_, DB, SK>,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
        let timeout_height = tx.get_timeout_height();

        // timeout_height of zero means no timeout height
        if timeout_height == 0 {
            return Ok(());
        }

        let block_height = ctx.get_height();

        if ctx.get_height() > timeout_height {
            return Err(AppError::Timeout {
                timeout: timeout_height,
                current: block_height,
            });
        }

        Ok(())
    }

    fn validate_memo_ante_handler<DB: Database, M: Message>(
        &self,
        ctx: &Context<'_, '_, DB, SK>,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
        let max_memo_chars = self.auth_keeper.get_auth_params(ctx).max_memo_characters;
        let memo_length: u64 = tx
            .get_memo()
            .len()
            .try_into()
            .map_err(|_| AppError::Memo(max_memo_chars))?;

        if memo_length > max_memo_chars {
            return Err(AppError::Memo(max_memo_chars));
        };
        Ok(())
    }

    fn deduct_fee_ante_handler<DB: Database, M: Message>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
        let fee = tx.get_fee();
        let fee_payer = tx.get_fee_payer();

        if !self.auth_keeper.has_account(ctx, fee_payer) {
            return Err(AppError::AccountNotFound);
        }

        if let Some(fee) = fee {
            self.bank_keeper.send_coins_from_account_to_module(
                ctx,
                fee_payer.to_owned(),
                Module::FeeCollector,
                fee.to_owned(),
            )?;
        }

        Ok(())
    }

    fn set_pub_key_ante_handler<DB: Database, M: Message>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
        let public_keys = tx.get_public_keys();
        let signers = tx.get_signers();

        // additional check not found in the sdk - this prevents a panic
        if signers.len() != public_keys.len() {
            return Err(AppError::TxValidation(format!(
                "wrong number of signer info; expected {}, got {}",
                signers.len(),
                public_keys.len()
            )));
        }

        for (i, key) in public_keys.into_iter().enumerate() {
            if let Some(key) = key {
                let addr = key.get_address();

                if &addr != signers[i] {
                    return Err(AppError::InvalidPublicKey);
                }

                let mut acct = self
                    .auth_keeper
                    .get_account(ctx, &addr)
                    .ok_or(AppError::AccountNotFound)?;

                if acct.get_public_key().is_some() {
                    continue;
                }

                acct.set_public_key(key.clone());
                self.auth_keeper.set_account(ctx, acct)
            }
        }

        Ok(())
    }

    fn sig_verification_handler<DB: Database, M: Message>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        tx: &TxWithRaw<M>,
    ) -> Result<(), AppError> {
        let signers = tx.tx.get_signers();
        let signature_data = tx.tx.get_signatures_data();

        // NOTE: this is also checked in validate_basic_ante_handler
        if signature_data.len() != signers.len() {
            return Err(AppError::TxValidation(format!(
                "wrong number of signatures; expected {}, got {}",
                signers.len(),
                signature_data.len()
            )));
        }

        for (i, signature_data) in signature_data.iter().enumerate() {
            let signer = signers[i];

            // check sequence number
            let acct = self
                .auth_keeper
                .get_account(ctx, signer)
                .ok_or(AppError::AccountNotFound)?;
            let account_seq = acct.get_sequence();
            if account_seq != signature_data.sequence {
                return Err(AppError::TxValidation(format!(
                    "incorrect tx sequence; expected {}, got {}",
                    account_seq, signature_data.sequence
                )));
            }

            // check signature
            let sign_bytes = SignDoc {
                body_bytes: tx.raw.body_bytes.clone(),
                auth_info_bytes: tx.raw.auth_info_bytes.clone(),
                chain_id: ctx.get_chain_id().to_owned(),
                account_number: acct.get_account_number(),
            }
            .encode_to_vec();
            let message = secp256k1::Message::from_hashed_data::<sha256::Hash>(&sign_bytes);

            let public_key = acct
                .get_public_key()
                .as_ref()
                .expect("account pub keys are set in set_pub_key_ante_handler"); //TODO: but can't they be set to None?

            //TODO: move sig verification into PublicKey
            match public_key {
                PublicKey::Secp256k1(pub_key) => {
                    let public_key =
                        Secp256k1PubKey::from_slice(&Vec::from(pub_key.to_owned())).unwrap(); //TODO: remove unwrap

                    let signature =
                        ecdsa::Signature::from_compact(&signature_data.signature).unwrap(); //TODO: remove unwrap

                    Secp256k1::verification_only()
                        .verify_ecdsa(&message, &signature, &public_key) //TODO: lib cannot be used for bitcoin sig verification
                        .map_err(|_| AppError::TxValidation("invalid signature".to_string()))?;
                }
            }
        }

        Ok(())
    }

    fn increment_sequence_ante_handler<DB: Database, M: Message>(
        &self,
        ctx: &mut Context<'_, '_, DB, SK>,
        tx: &Tx<M>,
    ) -> Result<(), AppError> {
        for signer in tx.get_signers() {
            let mut acct = self
                .auth_keeper
                .get_account(ctx, signer)
                .ok_or(AppError::AccountNotFound)?;
            acct.increment_sequence();
            self.auth_keeper.set_account(ctx, acct)
        }

        Ok(())
    }
}

// TODO: uncomment tests
// #[cfg(test)]
// mod tests {
//     use database::MemDB;
//     use proto_messages::cosmos::auth::v1beta1::{Account, BaseAccount};
//     use proto_types::AccAddress;

//     use crate::store::MultiStore;
//     use crate::types::tests::get_signed_tx;
//     use crate::types::InitContext;
//     use crate::x::auth::Auth;

//     use super::*;

//     #[test]
//     fn sig_verification_handler_works() {
//         // TODO: add tests for transactions that are expected to fail
//         let tx = get_signed_tx();

//         let db = MemDB::new();
//         let mut store = MultiStore::new(db);
//         let mut ctx = InitContext::new(&mut store, 0, "unit-testing".into());
//         let account = BaseAccount {
//             address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
//                 .unwrap(),
//             pub_key: None,
//             account_number: 1,
//             sequence: 1,
//         };
//         Auth::set_account(&mut ctx.as_any(), Account::Base(account));
//         set_pub_key_ante_handler(&mut ctx.as_any(), &tx).unwrap();
//         sig_verification_handler(&mut ctx.as_any(), &tx).unwrap();
//     }
// }
