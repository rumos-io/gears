fn main() {}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, gears::derive::ParamsKeys)]
pub enum GaiaParamsStoreKey {
    #[pkey(to_string = "bank/")]
    Bank,
    #[pkey(to_string = "bank/")]
    Auth,
    #[pkey(to_string = "baseapp/")]
    BaseApp,
}
