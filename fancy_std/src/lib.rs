mod ffi;
mod runtime;
mod r#yield;
pub mod time;

use std::future::Future;
use runtime::RUNTIME;

pub use r#yield::*;
pub use runtime::JoinHandle;

pub use fancy_std_macro::async_main as main;

pub fn spawn<R: 'static>(future: impl Future<Output = R> + 'static) -> JoinHandle<R> {
    RUNTIME.with(|rt| rt.spawn(future))
}
