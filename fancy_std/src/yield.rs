use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub fn yield_now() -> Yield {
    Yield::new()
}

pub struct Yield(pub(crate) bool);

impl Yield {
    pub fn new() -> Self {
        Self(true)
    }
}

impl Default for Yield {
    fn default() -> Self {
        Self::new()
    }
}

impl Future for Yield {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0 {
            false => Poll::Ready(()),
            true => {
                self.0 = false;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
