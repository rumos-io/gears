use std::marker::PhantomData;

use gears::{
    application::keepers::params::ParamsKeeper,
    derive::{AppMessage, Protobuf, Raw},
    params::ParamsSubspaceKey,
    store::StoreKey,
};
use ibc_proto::google::protobuf::Any;
use prost::Message;
use serde::{Deserialize, Serialize};

use super::handler::{ProposalHandler, ProposalHandlingError};

#[derive(Debug, Clone, PartialEq, Raw, Protobuf, AppMessage, Deserialize)]
#[raw(derive(Serialize, Deserialize, Clone, PartialEq))]
#[msg(url = "/cosmos.params.v1beta1/ParamChange")]
pub struct ParamChange<PSK: ParamsSubspaceKey> {
    #[raw(kind(string), raw = String)]
    #[proto(
        from = "PSK::from_subspace_str",
        from_ref,
        into = "PSK::name",
        into_ref
    )]
    pub subspace: PSK,
    #[raw(kind(bytes))]
    #[proto(repeated)]
    pub key: Vec<u8>,
    #[raw(kind(bytes))]
    #[proto(repeated)]
    pub value: Vec<u8>,
}

// Serde macro slightly dumb for such cases so I did it myself
impl<PSK: ParamsSubspaceKey> serde::Serialize for ParamChange<PSK> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        RawParamChange::from(self.clone()).serialize(serializer)
    }
}

#[derive(Debug, Clone, PartialEq, Raw, Protobuf, AppMessage, Deserialize)]
#[raw(derive(Serialize, Deserialize, Clone, PartialEq))]
#[msg(url = "/cosmos.params.v1beta1/ParameterChangeProposal")]
pub struct ParameterChangeProposal<PSK: ParamsSubspaceKey> {
    #[raw(kind(string), raw = String)]
    pub title: String,
    #[raw(kind(string), raw = String)]
    pub description: String,
    #[raw(kind(message), raw = RawParamChange, repeated)]
    #[proto(repeated)]
    pub changes: Vec<ParamChange<PSK>>,
}

impl<PSK: ParamsSubspaceKey> Serialize for ParameterChangeProposal<PSK> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        RawParameterChangeProposal::from(self.clone()).serialize(serializer)
    }
}

impl From<RawParameterChangeProposal> for Any {
    fn from(msg: RawParameterChangeProposal) -> Self {
        Any {
            type_url: "/cosmos.params.v1beta1/ParameterChangeProposal".to_owned(),
            value: msg.encode_to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParamChangeProposalHandler<PK, SK, PSK>(PhantomData<(PK, SK, PSK)>);

impl<PK, SK, PSK> Default for ParamChangeProposalHandler<PK, SK, PSK> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PK, SK, PSK> ParamChangeProposalHandler<PK, SK, PSK> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<PSK: ParamsSubspaceKey, PK: ParamsKeeper<PSK>, SK: StoreKey>
    ProposalHandler<ParameterChangeProposal<PSK>, SK> for ParamChangeProposalHandler<PK, SK, PSK>
{
    fn handle<
        CTX: gears::context::InfallibleContextMut<DB, SK>,
        DB: gears::store::database::Database,
    >(
        &self,
        ParameterChangeProposal {
            title: _,
            description: _,
            changes,
        }: ParameterChangeProposal<PSK>,
        ctx: &mut CTX,
    ) -> Result<(), super::handler::ProposalHandlingError> {
        for ParamChange {
            subspace,
            key,
            value,
        } in changes
        {
            if !PK::check_key(&key) {
                Err(ProposalHandlingError::KeyNotFound)?
            }

            if !PK::validate(&key, &value) {
                Err(ProposalHandlingError::InvalidProposal)?
            }

            let mut store = gears::params::gas::subspace_mut(ctx, &subspace);

            store.raw_key_set(key, value)?;
        }

        Ok(())
    }

    fn check(
        ParameterChangeProposal {
            title: _,
            description: _,
            changes,
        }: &ParameterChangeProposal<PSK>,
    ) -> bool {
        changes
            .iter()
            .all(|this| PK::check_key(&this.key) && PK::validate(&this.key, &this.value))
    }
}
