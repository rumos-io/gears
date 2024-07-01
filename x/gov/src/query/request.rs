use gears::{
    core::errors::CoreError, rest::Pagination, tendermint::types::proto::Protobuf,
    types::address::AccAddress,
};

use crate::types::proposal::ProposalStatus;

pub mod inner {
    pub use ibc_proto::cosmos::gov::v1beta1::QueryDepositRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryDepositsRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryParamsRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryProposalRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryProposalsRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryTallyResultRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryVoteRequest;
    pub use ibc_proto::cosmos::gov::v1beta1::QueryVotesRequest;
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryProposalRequest {
    pub proposal_id: u64,
}

impl TryFrom<inner::QueryProposalRequest> for QueryProposalRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryProposalRequest { proposal_id }: inner::QueryProposalRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self { proposal_id })
    }
}

impl From<QueryProposalRequest> for inner::QueryProposalRequest {
    fn from(QueryProposalRequest { proposal_id }: QueryProposalRequest) -> Self {
        Self { proposal_id }
    }
}

impl Protobuf<inner::QueryProposalRequest> for QueryProposalRequest {}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryProposalsRequest {
    pub voter: AccAddress,
    pub depositor: AccAddress,
    pub proposal_status: ProposalStatus,
    pub pagination: Option<Pagination>,
}

impl TryFrom<inner::QueryProposalsRequest> for QueryProposalsRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryProposalsRequest {
            proposal_status,
            voter,
            depositor,
            pagination,
        }: inner::QueryProposalsRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            voter: AccAddress::from_bech32(&voter)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            depositor: AccAddress::from_bech32(&depositor)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
            proposal_status: proposal_status.try_into()?,
            pagination: match pagination {
                Some(var) => Some(var.into()),
                None => None,
            },
        })
    }
}

impl From<QueryProposalsRequest> for inner::QueryProposalsRequest {
    fn from(
        QueryProposalsRequest {
            voter,
            depositor,
            proposal_status,
            pagination: _,
        }: QueryProposalsRequest,
    ) -> Self {
        Self {
            proposal_status: proposal_status as i32,
            voter: voter.to_string(),
            depositor: depositor.to_string(),
            pagination: None,
        }
    }
}

impl Protobuf<inner::QueryProposalsRequest> for QueryProposalsRequest {}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryVoteRequest {
    pub proposal_id: u64,
    pub voter: AccAddress,
}

impl TryFrom<inner::QueryVoteRequest> for QueryVoteRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryVoteRequest { proposal_id, voter }: inner::QueryVoteRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            voter: AccAddress::from_bech32(&voter)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
        })
    }
}

impl From<QueryVoteRequest> for inner::QueryVoteRequest {
    fn from(QueryVoteRequest { proposal_id, voter }: QueryVoteRequest) -> Self {
        Self {
            proposal_id,
            voter: voter.to_string(),
        }
    }
}

impl Protobuf<inner::QueryVoteRequest> for QueryVoteRequest {}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryVotesRequest {
    pub proposal_id: u64,
    pub pagination: Option<Pagination>,
}

impl TryFrom<inner::QueryVotesRequest> for QueryVotesRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryVotesRequest {
            proposal_id,
            pagination,
        }: inner::QueryVotesRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            pagination: pagination.map(|e| e.into()),
        })
    }
}

impl From<QueryVotesRequest> for inner::QueryVotesRequest {
    fn from(
        QueryVotesRequest {
            proposal_id,
            pagination: _,
        }: QueryVotesRequest,
    ) -> Self {
        Self {
            proposal_id,
            pagination: None,
        }
    }
}

impl Protobuf<inner::QueryVotesRequest> for QueryVotesRequest {}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryParamsRequest(pub ParamsQuery);

#[derive(Clone, PartialEq, Debug, strum::EnumString, strum::Display)]
pub enum ParamsQuery {
    #[strum(serialize = "tallying")]
    Tally,
    #[strum(serialize = "voting")]
    Voting,
    #[strum(serialize = "deposit")]
    Deposit,
}

impl TryFrom<inner::QueryParamsRequest> for QueryParamsRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryParamsRequest { params_type }: inner::QueryParamsRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self(params_type.parse().map_err(|_| {
            CoreError::DecodeGeneral("failed to parse `params_type`".to_owned())
        })?))
    }
}

impl From<QueryParamsRequest> for inner::QueryParamsRequest {
    fn from(value: QueryParamsRequest) -> Self {
        Self {
            params_type: value.0.to_string(),
        }
    }
}

impl Protobuf<inner::QueryParamsRequest> for QueryParamsRequest {}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryDepositRequest {
    pub proposal_id: u64,
    pub depositor: AccAddress,
}

impl TryFrom<inner::QueryDepositRequest> for QueryDepositRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryDepositRequest {
            proposal_id,
            depositor,
        }: inner::QueryDepositRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            depositor: AccAddress::from_bech32(&depositor)
                .map_err(|e| CoreError::DecodeAddress(e.to_string()))?,
        })
    }
}

impl From<QueryDepositRequest> for inner::QueryDepositRequest {
    fn from(
        QueryDepositRequest {
            proposal_id,
            depositor,
        }: QueryDepositRequest,
    ) -> Self {
        Self {
            proposal_id,
            depositor: depositor.to_string(),
        }
    }
}

impl Protobuf<inner::QueryDepositRequest> for QueryDepositRequest {}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryDepositsRequest {
    pub proposal_id: u64,
    pub pagination: Option<Pagination>,
}

impl TryFrom<inner::QueryDepositsRequest> for QueryDepositsRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryDepositsRequest {
            proposal_id,
            pagination,
        }: inner::QueryDepositsRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id,
            pagination: pagination.map(|e| e.into()),
        })
    }
}

impl From<QueryDepositsRequest> for inner::QueryDepositsRequest {
    fn from(
        QueryDepositsRequest {
            proposal_id,
            pagination: _,
        }: QueryDepositsRequest,
    ) -> Self {
        Self {
            proposal_id,
            pagination: None,
        }
    }
}

impl Protobuf<inner::QueryDepositsRequest> for QueryDepositsRequest {}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryTallyResultRequest {
    pub proposal_id: u64,
}

impl TryFrom<inner::QueryTallyResultRequest> for QueryTallyResultRequest {
    type Error = CoreError;

    fn try_from(
        inner::QueryTallyResultRequest { proposal_id }: inner::QueryTallyResultRequest,
    ) -> Result<Self, Self::Error> {
        Ok(Self { proposal_id })
    }
}

impl From<QueryTallyResultRequest> for inner::QueryTallyResultRequest {
    fn from(QueryTallyResultRequest { proposal_id }: QueryTallyResultRequest) -> Self {
        Self { proposal_id }
    }
}

impl Protobuf<inner::QueryTallyResultRequest> for QueryTallyResultRequest {}
