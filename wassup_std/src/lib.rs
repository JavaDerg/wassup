mod ffi;
mod runtime;
pub mod time;
mod r#yield;

use runtime::RUNTIME;
use std::future::Future;

pub use r#yield::*;
pub use runtime::JoinHandle;
pub use tracing::{self, debug, error, info, trace, warn};

pub use wassup_std_macros::async_main as main;

pub fn spawn<R: 'static>(future: impl Future<Output = R> + 'static) -> JoinHandle<R> {
    RUNTIME.with(|rt| rt.spawn(future))
}

#[doc(hidden)]
pub fn startup_runtime() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::level_filters::LevelFilter::TRACE)
        .init();

    trace!("Hello from wassup_std!");
    debug!("Hello from wassup_std!");
    info!("Hello from wassup_std!");
    warn!("Hello from wassup_std!");
    error!("Hello from wassup_std!");
}

#[doc(hidden)]
pub fn shutdown_runtime() {
    RUNTIME.with(|rt| rt.shutdown());
}
