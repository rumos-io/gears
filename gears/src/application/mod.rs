pub mod app;
pub mod client;
pub mod command;
pub mod handlers;

pub trait ApplicationInfo: Clone + Sync + Send + 'static {
    const APP_NAME: &'static str;
    const APP_VERSION: &'static str;

    fn home_dir() -> std::path::PathBuf {
        dirs::home_dir()
            .expect("failed to get home dir")
            .join(format!(".{}", Self::APP_NAME)) // TODO: what about using version as prefix?
    }
}
