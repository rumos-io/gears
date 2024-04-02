use database::Database;
use gears::{
    types::context::{tx_context::TxContext, ContextMut},
    x::params::ParamsSubspaceKey,
};
use proto_messages::{
    any::{Any, PrimitiveAny},
    cosmos::ibc::types::{
        core::{
            client::context::{
                client_state::{ClientStateCommon, ClientStateExecution, ClientStateValidation},
                types::{
                    events::{
                        CLIENT_ID_ATTRIBUTE_KEY, CLIENT_MISBEHAVIOUR_EVENT,
                        CLIENT_TYPE_ATTRIBUTE_KEY, CONSENSUS_HEIGHTS_ATTRIBUTE_KEY,
                        CONSENSUS_HEIGHT_ATTRIBUTE_KEY, CREATE_CLIENT_EVENT, UPDATE_CLIENT_EVENT,
                        UPGRADE_CLIENT_EVENT,
                    },
                    Status, UpdateKind,
                },
            },
            commitment::{CommitmentProofBytes, CommitmentRoot},
            host::identifiers::{ClientId, ClientType},
        },
        tendermint::{
            consensus_state::WrappedConsensusState, informal::Event, WrappedTendermintClientState,
        },
    },
};
use store::StoreKey;

use crate::{
    errors::tx::client::{
        ClientCreateError, ClientRecoverError, ClientUpdateError, ClientUpgradeError,
    },
    params::{self, AbciParamsKeeper},
    types::ContextShim,
};

use super::{client_state_get, params_get};

#[derive(Debug, Clone)]
pub struct TxKeeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    params_keeper: AbciParamsKeeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> TxKeeper<SK, PSK> {
    pub fn new(
        store_key: SK,
        params_keeper: gears::x::params::Keeper<SK, PSK>,
        params_subspace_key: PSK,
    ) -> Self {
        let abci_params_keeper = AbciParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        TxKeeper {
            store_key,
            params_keeper: abci_params_keeper,
        }
    }

    pub fn recover_client<DB: Database + Send + Sync>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        subject_client_id: &ClientId,
        substitute_client_id: &ClientId,
    ) -> Result<(), ClientRecoverError> {
        let subj_client_state = client_state_get(&self.store_key, ctx, subject_client_id)?;
        {
            let mut shim_ctx = ContextShim::new(ctx, self.store_key.clone());
            let subj_client_status = subj_client_state.status(&mut shim_ctx, subject_client_id)?;
            if subj_client_status == Status::Active {
                return Err(ClientRecoverError::SubjectStatus {
                    client_id: subject_client_id.clone(),
                    status: subj_client_status,
                });
            }
        }

        let subs_client_state = client_state_get(&self.store_key, ctx, substitute_client_id)?;
        if subj_client_state.latest_height() >= subs_client_state.latest_height() {
            return Err(ClientRecoverError::InvalidHeight {
                subject: subj_client_state.latest_height(),
                substitute: subs_client_state.latest_height(),
            });
        }

        {
            let mut shim_ctx = ContextShim::new(ctx, self.store_key.clone());
            let subs_client_status =
                subj_client_state.status(&mut shim_ctx, substitute_client_id)?;
            if subs_client_status != Status::Active {
                return Err(ClientRecoverError::SubstituteStatus {
                    client_id: substitute_client_id.clone(),
                    status: subs_client_status,
                });
            }
        }

        // if err := subjectClientState.CheckSubstituteAndUpdateState(ctx, k.cdc, subjectClientStore, substituteClientStore, substituteClientState); err != nil {
        //     return errorsmod.Wrap(err, "failed to validate substitute client")
        // }

        ctx.append_events(vec![
            Event::new(
                UPGRADE_CLIENT_EVENT,
                [
                    ("subject_client_id", subject_client_id.as_str().to_owned()),
                    (
                        CLIENT_TYPE_ATTRIBUTE_KEY,
                        subs_client_state.client_type().as_str().to_owned(),
                    ),
                ],
            ),
            Event::new(
                "message",
                [
                    (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO: const
                ],
            ),
        ]);

        Ok(())
    }

    pub fn client_upgrade<DB: Database + Send + Sync>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        client_id: &ClientId,
        upgraded_client_state: WrappedTendermintClientState,
        upgraded_consensus_state: WrappedConsensusState,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
    ) -> Result<(), ClientUpgradeError> {
        let client_state = client_state_get(&self.store_key, ctx, client_id)?;

        let mut shim_ctx = ContextShim::new(ctx, self.store_key.clone());
        let client_status = client_state.status(&mut shim_ctx, client_id)?;

        if client_status != Status::Active {
            return Err(ClientUpgradeError::NotActive {
                client_id: client_id.clone(),
                status: client_status,
            });
        }

        let last_height = client_state.latest_height();

        {
            let upgraded_height = upgraded_client_state.latest_height();

            if upgraded_height <= last_height {
                return Err(ClientUpgradeError::HeightError {
                    upgraded: upgraded_height,
                    current: last_height,
                });
            }
        }

        let root_hash = CommitmentRoot::from_bytes(&self.root_hash(ctx));
        client_state.verify_upgrade_client(
            upgraded_client_state.into(),
            upgraded_consensus_state.into(),
            proof_upgrade_client,
            proof_upgrade_consensus_state,
            &root_hash,
        )?;

        ctx.append_events(vec![
            Event::new(
                UPGRADE_CLIENT_EVENT,
                [
                    (CLIENT_ID_ATTRIBUTE_KEY, client_id.as_str().to_owned()),
                    (
                        CLIENT_TYPE_ATTRIBUTE_KEY,
                        client_state.client_type().as_str().to_owned(),
                    ),
                    (
                        CONSENSUS_HEIGHT_ATTRIBUTE_KEY,
                        client_state.latest_height().to_string(),
                    ),
                ],
            ),
            Event::new(
                "message",
                [
                    (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO: const
                ],
            ),
        ]);

        Ok(())
    }

    pub fn client_update<DB: Database + Send + Sync>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientUpdateError> {
        let client_state = client_state_get(&self.store_key, ctx, client_id)?;
        let client_type = client_state.client_type();
        let params = params_get(&self.params_keeper, ctx)?;

        let client_message = PrimitiveAny::from(client_message);

        let mut shim_ctx = ContextShim::new(ctx, self.store_key.clone());

        let client_status = if params.is_client_allowed(&client_type) {
            Status::Unauthorized
        } else {
            client_state.status(&mut shim_ctx, client_id)?
        };

        if client_status != Status::Active {
            return Err(ClientUpdateError::NotActive {
                client_id: client_id.clone(),
                status: client_status,
            });
        }

        client_state.verify_client_message(
            &shim_ctx,
            client_id,
            client_message.clone(),
            &UpdateKind::UpdateClient,
        )?;
        let misbehaviour = client_state.check_for_misbehaviour(
            &shim_ctx,
            client_id,
            client_message.clone(),
            &UpdateKind::UpdateClient,
        )?;
        if misbehaviour {
            client_state.update_state_on_misbehaviour(
                &mut shim_ctx,
                client_id,
                client_message,
                &UpdateKind::UpdateClient,
            )?;

            ctx.append_events(vec![
                Event::new(
                    CLIENT_MISBEHAVIOUR_EVENT,
                    [
                        (CLIENT_ID_ATTRIBUTE_KEY, client_id.as_str().to_owned()),
                        (CLIENT_TYPE_ATTRIBUTE_KEY, client_type.as_str().to_owned()),
                    ],
                ),
                Event::new(
                    "message",
                    [
                        (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO: const
                    ],
                ),
            ]);

            Ok(())
        } else {
            let height = client_state.update_state(&mut shim_ctx, client_id, client_message)?;
            let consensus_state_height = height
                .first()
                .map(|this| this.to_string())
                .unwrap_or(String::new());

            let mut heights = String::new();
            height
                .into_iter()
                .for_each(|this| heights.push_str(&this.to_string()));

            ctx.append_events(vec![
                Event::new(
                    UPDATE_CLIENT_EVENT,
                    [
                        (CLIENT_ID_ATTRIBUTE_KEY, client_id.as_str().to_owned()),
                        (CLIENT_TYPE_ATTRIBUTE_KEY, client_type.as_str().to_owned()),
                        (CONSENSUS_HEIGHT_ATTRIBUTE_KEY, consensus_state_height), // NOTE: Deprecated: so it should be removed
                        (CONSENSUS_HEIGHTS_ATTRIBUTE_KEY, heights),
                    ],
                ),
                Event::new(
                    "message",
                    [
                        (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO: const
                    ],
                ),
            ]);

            Ok(())
        }
    }

    pub fn client_create<'a, 'b, DB: Database + Send + Sync>(
        &self,
        ctx: &'a mut TxContext<'b, DB, SK>,
        client_state: &(impl ClientStateCommon
              + ClientStateExecution<ContextShim<'a, 'b, DB, SK>>
              + ClientStateValidation<ContextShim<'a, 'b, DB, SK>>),
        consensus_state: WrappedConsensusState,
    ) -> Result<ClientId, ClientCreateError> {
        let client_type = client_state.client_type();
        if client_type
            == ClientType::new("09-localhost")
                .expect("Unreachable: localhost should be valid client type")
        {
            return Err(ClientCreateError::InvalidType(client_state.client_type()));
        }

        let params = params_get(&self.params_keeper, ctx)?;

        if !params.is_client_allowed(&client_type) {
            return Err(ClientCreateError::NotAllowed(client_type));
        }

        let client_id = self.client_indentifier_generate(ctx, &client_type)?;

        // TODO: Is this okay to create events before rest of code?
        ctx.append_events(vec![
            Event::new(
                CREATE_CLIENT_EVENT,
                [
                    (CLIENT_ID_ATTRIBUTE_KEY, client_id.as_str().to_owned()),
                    (CLIENT_TYPE_ATTRIBUTE_KEY, client_type.as_str().to_owned()),
                    (
                        CONSENSUS_HEIGHT_ATTRIBUTE_KEY,
                        client_state.latest_height().to_string(),
                    ),
                ],
            ),
            Event::new(
                "message",
                [
                    (crate::types::ATTRIBUTE_KEY_MODULE, "ibc_client"), // TODO: const
                ],
            ),
        ]);

        // FIXME: fix lifetimes so borrow checker would be happy with this code before events
        let mut ctx = ContextShim::new(ctx, self.store_key.clone());

        client_state.initialise(&mut ctx, &client_id, consensus_state.into())?;
        client_state.status(&mut ctx, &client_id)?;

        Ok(client_id)
    }

    fn client_indentifier_generate<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        client_type: &ClientType,
    ) -> Result<ClientId, ClientCreateError> {
        let next_client_seq = self.next_client_sequence_get(ctx)?;

        self.next_client_sequence_set(ctx, next_client_seq + 1);

        ClientId::new(client_type.as_str(), next_client_seq)
            .map_err(ClientCreateError::IdentifierError)
    }

    fn next_client_sequence_set<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        sequence: u64,
    ) {
        self.params_keeper.set(
            ctx,
            params::NEXT_CLIENT_SEQUENCE.as_bytes().iter().cloned(),
            sequence.to_be_bytes(),
        )
    }

    fn next_client_sequence_get<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
    ) -> Result<u64, ClientCreateError> {
        let bytes = self.params_keeper.get(ctx, &params::NEXT_CLIENT_SEQUENCE)?;

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

    fn root_hash<DB: Database>(&self, ctx: &mut TxContext<'_, DB, SK>) -> [u8; 32] {
        ctx.get_kv_store(&self.store_key).head_commit_hash()
    }
}
