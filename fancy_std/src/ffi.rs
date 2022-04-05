use crate::runtime::RUNTIME;

type Duration = u64;

extern "C" {
    pub static yield_rt: u32;

    pub fn wake();

    pub fn shutdown_rt() -> !;
}

#[no_mangle]
pub extern "C" fn poll_runtime() -> Duration {
    let dur = RUNTIME.with(|rt| {
        loop {
            let next = rt.poll();
            if next != 0 || unsafe { yield_rt } != 0 {
                break next;
            }
        }
    });
    dur
}
