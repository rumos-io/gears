use std::path::PathBuf;

use crate::app::APP_NAME;

pub fn get_default_home_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|mut h| {
        h.push(format!(".{}", APP_NAME));
        h
    })
}
