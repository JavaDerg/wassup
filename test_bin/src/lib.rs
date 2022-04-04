use std::io::Write;
use std::time::{Duration, Instant, UNIX_EPOCH};
use fancy_std::time::sleep_for;
use fancy_std::{log, yield_now};


#[fancy_std::main]
pub async fn main() {
    log(UNIX_EPOCH.elapsed().unwrap().as_secs());
    log(0xDEAD0001);
    for i in 0.. {
        log(0xDEAD0000 + i);
        println!("{}", i);
        sleep_for(Duration::from_secs(1)).await
    }
    log(0xDEAD00FF);
}
