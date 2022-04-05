use fancy_std::time::sleep_for;
use fancy_std::{info, spawn, yield_now};
use std::io::Write;
use std::time::{Duration, Instant, UNIX_EPOCH};

#[fancy_std::main]
pub async fn main() {
    let mut handles = vec![];
    for n in 0..1 {
        handles.push(spawn(count(n)));
        // sleep_for(Duration::from_millis(100)).await;
    }
    for handle in handles {
        handle.await;
    }
}

pub async fn count(n: u32) {
    let start = Instant::now();
    for i in 0.. {
        info!("id={n}; step={i}; elapsed={:?}", start.elapsed());
        sleep_for(Duration::from_millis(1000)).await;
    }
}
