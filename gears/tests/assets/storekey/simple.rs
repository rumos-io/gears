#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, gears::derive::StoreKeys)]
#[skey(params = Params)]
pub enum GaiaStoreKey {
    #[skey(store_str = "bank")]
    Bank,
    #[skey(store_str = "acc")]
    Auth,
    #[skey(store_str = "params")]
    Params,
}

fn main() {}
