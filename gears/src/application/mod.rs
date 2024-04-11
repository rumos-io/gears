pub mod client;
pub mod handlers;
pub mod node;

pub trait ApplicationInfo: Clone + Sync + Send + 'static {
    const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
    const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn home_dir() -> std::path::PathBuf {
        dirs::home_dir()
            .expect("failed to get home dir")
            .join(format!(".{}", Self::APP_NAME)) // TODO: what about using version as prefix?
    }
}
