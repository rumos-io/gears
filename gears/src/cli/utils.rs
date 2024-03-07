use rand::distributions::DistString;

use crate::ApplicationInfo;

pub(super) fn home_dir<T: ApplicationInfo>() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("failed to get home dir")
        .join(T::APP_NAME)
}

pub(super) const RAND_LENGTH: usize = 10;

pub(super) fn rand_string() -> String {
    rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), RAND_LENGTH)
}
