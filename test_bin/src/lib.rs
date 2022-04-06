use std::time::{Duration, Instant};
use wassup_std::{info, spawn};
use wassup_std::time::sleep_for;

#[wassup_std::main]
pub async fn main() {
    let start = Instant::now();
    let mut handles = vec![];
    for n in 0..10 {
        handles.push(spawn(count(n, start)));
        sleep_for(Duration::from_millis(100)).await;
    }
    for handle in handles {
        handle.await;
    }
}

pub async fn count(n: u32, start: Instant) {
    for i in 0.. {
        info!("id={n}; step={i}; elapsed={:?}", start.elapsed());
        sleep_for(Duration::from_millis(1000)).await;
    }
}
