use crate::log;
use crate::runtime::RUNTIME;

type Duration = u64;

extern "C" {
    pub static yield_rt: u32;

    pub fn wake();

    pub fn log_n(n: u64);
    pub fn shutdown_rt() -> !;
}

#[no_mangle]
pub extern "C" fn poll_runtime() -> Duration {
    log(0x30);
    let dur = RUNTIME.with(|rt| {
        let next = rt.poll();
        log(0x31);
        next.as_micros() as u64
    });
    log(0x32);
    dur
}
