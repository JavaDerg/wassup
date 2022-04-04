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
    RUNTIME.with(|rt| {
        let next = rt.poll();
        next.as_micros() as u64
    })
}
