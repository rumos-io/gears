use super::*;
use crate::types::{
    QueryAllEvidenceRequest, QueryAllEvidenceResponse, QueryEvidenceRequest, QueryEvidenceResponse,
    RawEquivocation,
};
use gears::{
    context::query::QueryContext,
    ext::{IteratorPaginate, Pagination, PaginationKey},
};

impl<
        SK: StoreKey,
        StkK: SlashingStakingKeeper<SK, M>,
        SlsK: EvidenceSlashingKeeper<SK, M>,
        E: Evidence + Default,
        M: Module,
    > Keeper<SK, StkK, SlsK, E, M>
where
    <E as std::convert::TryFrom<Any>>::Error: std::fmt::Debug,
{
    pub fn query_evidence<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryEvidenceRequest,
    ) -> QueryEvidenceResponse {
        if let Some(evidence) = self
            .evidence_non_fallible::<QueryContext<DB, SK>, DB, E>(ctx, query.evidence_hash)
            .unwrap_gas()
        {
            QueryEvidenceResponse {
                evidence: Some(evidence.into()),
            }
        } else {
            let evidence = self
                .evidence_non_fallible::<QueryContext<DB, SK>, DB, RawEquivocation>(
                    ctx,
                    query.evidence_hash,
                )
                .unwrap_gas();

            QueryEvidenceResponse {
                evidence: evidence.map(Into::into),
            }
        }
    }

    pub fn query_all_evidence<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: QueryAllEvidenceRequest,
    ) -> QueryAllEvidenceResponse {
        #[derive(Clone)]
        struct EvidenceKeyWrapper(Any);
        impl PaginationKey for EvidenceKeyWrapper {
            fn iterator_key(&self) -> std::borrow::Cow<'_, [u8]> {
                // we have a type url and random bytes of type data
                // TODO: do we need to handle the data inside?
                // std::borrow::Cow::Borrowed(&self.0.value)
                // if yes, then we have to sort resulted vector
                std::borrow::Cow::Borrowed(self.0.type_url.as_bytes())
            }
        }

        let (pagination_result, iter) = self
            .all_evidence_non_fallible::<QueryContext<DB, SK>, DB, E>(ctx)
            .unwrap_gas()
            .into_iter()
            .map(Into::into)
            .map(EvidenceKeyWrapper)
            .chain(
                self.all_evidence_non_fallible::<QueryContext<DB, SK>, DB, RawEquivocation>(ctx)
                    .unwrap_gas()
                    .into_iter()
                    .map(Into::into)
                    .map(EvidenceKeyWrapper),
            )
            .paginate(Pagination::from(query.pagination));
        let evidence: Vec<Any> = iter.map(|k| k.0).collect();

        QueryAllEvidenceResponse {
            evidence,
            pagination: Some(pagination_result.into()),
        }
    }
}
