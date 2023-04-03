use database::DB;

use crate::{error::AppError, types::QueryContext};

#[derive(Debug, Clone)]
pub struct IBC {}

pub struct GenesisState {}

impl IBC {
    pub fn query_router() {}
    // pub fn query_client_states<T: DB>(
    //     ctx: &QueryContext<T>,
    //     req: QueryAllBalancesRequest,
    // ) -> Result<QueryAllBalancesResponse, AppError> {
    //     let bank_store = ctx.get_kv_store(Store::Bank);
    //     let prefix = create_denom_balance_prefix(req.address);
    //     let account_store = bank_store.get_immutable_prefix_store(prefix);

    //     let mut balances = vec![];

    //     for (_, coin) in account_store.range(..) {
    //         let coin: Coin = Coin::decode::<Bytes>(coin.to_owned().into())
    //             .expect("invalid data in database - possible database corruption");
    //         balances.push(coin);
    //     }

    //     return Ok(QueryAllBalancesResponse {
    //         balances,
    //         pagination: None,
    //     });
    // }
}
