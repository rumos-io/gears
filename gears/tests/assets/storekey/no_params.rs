#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, gears::derive::StoreKeys)]
pub enum GaiaStoreKey {
    #[skey(to_string = "bank")]
    Bank,
    #[skey(to_string = "acc")]
    Auth,
    #[skey(to_string = "params")]
    Params,
}

fn main() {}
