use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
