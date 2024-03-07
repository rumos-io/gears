use rand::distributions::DistString;

pub(super) const RAND_LENGTH: usize = 10;

pub(super) fn rand_string() -> String {
    rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), RAND_LENGTH)
}
