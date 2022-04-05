mod ffi;
mod runtime;
mod r#yield;
pub mod time;

use std::future::Future;
use runtime::RUNTIME;

pub use r#yield::*;
pub use runtime::JoinHandle;
pub use tracing::{self, trace, debug, info, warn, error};

pub use fancy_std_macro::async_main as main;

pub fn spawn<R: 'static>(future: impl Future<Output = R> + 'static) -> JoinHandle<R> {
    RUNTIME.with(|rt| rt.spawn(future))
}

#[doc(hidden)]
pub fn startup_runtime() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::level_filters::LevelFilter::TRACE)
        .init();

    trace!("Hello from fancy_std!");
    debug!("Hello from fancy_std!");
    info!("Hello from fancy_std!");
    warn!("Hello from fancy_std!");
    error!("Hello from fancy_std!");
}

#[doc(hidden)]
pub fn shutdown_runtime() {
    RUNTIME.with(|rt| rt.shutdown());
}
