use database::Database;
use gears::{types::context::init_context::InitContext, x::params::ParamsSubspaceKey};
use prost::Message;
use proto_messages::cosmos::ibc::{
    tx::MsgCreateClient,
    types::{ClientStateCommon, ClientType, IdentifierError, RawClientId},
};
use store::StoreKey;

use crate::params::{self, AbciParamsKeeper, Params, ParamsError, RawParams};

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    params_keeper: AbciParamsKeeper<SK, PSK>,
    // auth_keeper: auth::Keeper<SK, PSK>,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientCreateError {
    #[error("cannot create client of type: {0}")]
    InvalidType(ClientType),
    #[error("client state type {0} is not registered in the allowlist")]
    NotAllowed(ClientType),
    #[error("{0}")]
    ParamsError(#[from] ParamsError),
    #[error("Decode error: {0}")]
    DecodeError(#[from] prost::DecodeError),
    #[error("{0}")]
    IdentifierError(#[from] IdentifierError),
    #[error("Unexpected error: {0}")]
    CustomError(String),
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(
        store_key: SK,
        params_keeper: gears::x::params::Keeper<SK, PSK>,
        params_subspace_key: PSK,
    ) -> Self {
        let abci_params_keeper = AbciParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        Keeper {
            store_key,
            params_keeper: abci_params_keeper,
        }
    }

    pub fn client_create<DB: Database>(
        &mut self,
        ctx: &mut InitContext<'_, DB, SK>,
        msg: MsgCreateClient,
        state: impl ClientStateCommon,
    ) -> Result<(), ClientCreateError> {
        let client_type = state.client_type();
        if client_type
            == ClientType::new("09-localhost")
                .expect("Unreachable: localhost should be valid client type")
        {
            return Err(ClientCreateError::InvalidType(state.client_type()));
        }

        let params = self.params_get(ctx)?;

        if !params.is_client_allowed(&client_type) {
            return Err(ClientCreateError::NotAllowed(client_type));
        }

        let client_id = self.client_indentifier_generate(ctx, &client_type)?;

        Ok(())
    }

    fn client_indentifier_generate<DB: Database>(
        &mut self,
        ctx: &mut InitContext<'_, DB, SK>,
        client_type: &ClientType,
    ) -> Result<RawClientId, ClientCreateError> {
        let next_client_seq = self.next_client_sequence_get(ctx)?;

        self.next_client_sequence_set(ctx, next_client_seq + 1);

        RawClientId::new(client_type.as_str(), next_client_seq)
            .map_err(ClientCreateError::IdentifierError)
    }

    fn next_client_sequence_set<DB: Database>(
        &mut self,
        ctx: &mut InitContext<'_, DB, SK>,
        sequence: u64,
    ) {
        let mut ctx = gears::types::context::context::Context::InitContext(ctx);
        self.params_keeper.set(
            &mut ctx,
            params::NEXT_CLIENT_SEQUENCE.as_bytes().into_iter().cloned(),
            sequence.to_be_bytes(),
        )
    }

    fn next_client_sequence_get<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
    ) -> Result<u64, ClientCreateError> {
        let ctx = gears::types::context::context::Context::InitContext(ctx);
        let bytes = self
            .params_keeper
            .get(&ctx, &params::NEXT_CLIENT_SEQUENCE)?;

        if bytes.is_empty() {
            Err(ClientCreateError::CustomError(
                "next client sequence is empty".to_owned(),
            ))?
        }

        // TODO: should return error if remains != empty or ignore?
        let (int_bytes, _remains) = bytes.split_at(std::mem::size_of::<u64>());

        Ok(u64::from_be_bytes(int_bytes.try_into().map_err(
            |e: std::array::TryFromSliceError| ClientCreateError::CustomError(e.to_string()),
        )?))
    }

    fn params_get<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, SK>,
    ) -> Result<Params, ClientCreateError> {
        let ctx = gears::types::context::context::Context::InitContext(ctx);
        let bytes = self.params_keeper.get(&ctx, &params::CLIENT_PARAMS_KEY)?;

        Ok(RawParams::decode(bytes.as_slice())?.into())
    }
}
