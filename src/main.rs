mod core;
mod utils;

use self::core::filter::filter;

fn main() {
    let s = std::time::Instant::now();
    filter(
        "./v3.18.44-en-sample.csv".to_string(),
        "filtered".to_string(),
    )
    .unwrap();
    println!("Elapsed: {:.2?}", s.elapsed());
}
