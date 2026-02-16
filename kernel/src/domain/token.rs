use rand::distr::Alphabetic;
use rand::{self, RngExt};
pub async fn generate_transfer_token() -> String {
    let rng = rand::rng();
    let rand_string: String = rng
        .sample_iter(&Alphabetic)
        .take(10)
        .map(char::from)
        .collect();
    return rand_string;
}
