use tendermint_rpc::endpoint::validators::DEFAULT_VALIDATORS_PER_PAGE;
use tendermint_rpc::{Error, HttpClient as TendermintHttpClient, SimpleRequest};

pub use tendermint_rpc::Client;
pub use tendermint_rpc::HttpClientUrl;
pub use tendermint_rpc::Paging;

#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: TendermintHttpClient,
}
impl HttpClient {
    pub fn new<U>(url: U) -> Result<Self, Error>
    where
        U: TryInto<HttpClientUrl, Error = Error>,
    {
        let client = TendermintHttpClient::new(url)?;
        Ok(Self { inner: client })
    }

    pub fn new_with_proxy<U, P>(url: U, proxy_url: P) -> Result<Self, Error>
    where
        U: TryInto<HttpClientUrl, Error = Error>,
        P: TryInto<HttpClientUrl, Error = Error>,
    {
        let client = TendermintHttpClient::new_with_proxy(url, proxy_url)?;
        Ok(Self { inner: client })
    }
}

#[async_trait::async_trait]
impl Client for HttpClient {
    async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: SimpleRequest,
    {
        self.inner.perform(request).await
    }
}

impl HttpClient {
    // TODO(thane): Simplify once validators endpoint removes pagination.
    /// `/validators`: get validators a given height.
    pub async fn validators_latest(
        &self,
        paging: Paging,
    ) -> Result<tendermint_rpc::endpoint::validators::Response, Error> {
        match paging {
            Paging::Default => {
                self.perform(tendermint_rpc::endpoint::validators::Request::new(
                    None, None, None,
                ))
                .await
            }
            Paging::Specific {
                page_number,
                per_page,
            } => {
                self.perform(tendermint_rpc::endpoint::validators::Request::new(
                    None,
                    Some(page_number),
                    Some(per_page),
                ))
                .await
            }
            Paging::All => {
                let mut page_num = 1_usize;
                let mut validators = Vec::new();
                let per_page = DEFAULT_VALIDATORS_PER_PAGE.into();
                loop {
                    let response = self
                        .perform(tendermint_rpc::endpoint::validators::Request::new(
                            None,
                            Some(page_num.into()),
                            Some(per_page),
                        ))
                        .await?;
                    validators.extend(response.validators);
                    if validators.len() as i32 == response.total {
                        return Ok(tendermint_rpc::endpoint::validators::Response::new(
                            response.block_height,
                            validators,
                            response.total,
                        ));
                    }
                    page_num += 1;
                }
            }
        }
    }
}
