use std::io::Write;
use std::time::{Duration, Instant, UNIX_EPOCH};
use fancy_std::time::sleep_for;
use fancy_std::{log, yield_now};


#[fancy_std::main]
pub async fn main() {
    let start = Instant::now();
    for i in 0.. {
        println!("step={}; elapsed={:?}", i, start.elapsed());
        sleep_for(Duration::from_millis(500)).await;
    }
}
