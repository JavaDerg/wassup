use std::io::Write;
use std::time::{Duration, Instant};
use fancy_std::time::sleep_for;
use fancy_std::{log, yield_now};


#[fancy_std::main]
pub async fn main() {
    log(0xDEADBEEF);
    for i in 0.. {
        println!("{}", i);
        sleep_for(Duration::from_secs(1)).await
    }
}
