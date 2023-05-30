use std::path::PathBuf;

pub fn get_default_home_dir(app_name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|mut h| {
        h.push(format!(".{}", app_name));
        h
    })
}
