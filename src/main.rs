mod core;
mod utils;

use self::core::filter::filter;

fn main() {
    filter(
        "./v3.18.44-en-sample.csv".to_string(),
        "filtered".to_string(),
    )
    .unwrap();
}
