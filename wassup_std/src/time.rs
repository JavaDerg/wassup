use crate::runtime::{SleepHandle, RUNTIME};
use std::future::Future;

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub struct Sleep {
    until: Instant,
    handle: Option<SleepHandle>,
}

pub fn sleep_until(until: Instant) -> Sleep {
    Sleep {
        until,
        handle: None,
    }
}

pub fn sleep_for(duration: Duration) -> Sleep {
    sleep_until(Instant::now() + duration)
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.until <= Instant::now() {
            return Poll::Ready(());
        } else {
            let handle = RUNTIME.with(|rt| rt.schedule_sleep(self.until, cx.waker().clone()));
            self.handle.replace(handle);
        }
        Poll::Pending
    }
}
