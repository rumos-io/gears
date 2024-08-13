fn main() {}

#[derive(strum::EnumIter, Debug, PartialEq, Eq, Hash, Clone, gears::derive::ParamsKeys)]
pub enum GaiaParamsStoreKey {
    #[pkey(prefix_str = "bank/")]
    Bank,
    #[pkey(prefix_str = "auth/")]
    Auth,
    #[pkey(prefix_str = "baseapp/")]
    BaseApp,
}
