use gears::application::ApplicationInfo;

#[derive(Debug, Clone)]
pub struct GaiaApplication;

impl ApplicationInfo for GaiaApplication {
    const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
    const APP_VERSION: &'static str = env!("GIT_HASH");
}
