use crate::{ffi, shutdown_runtime, Yield};
use std::any::Any;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::future::Future;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::time::{Duration, Instant};

static IN_RUNTIME: AtomicBool = AtomicBool::new(false);
static TASK_ID: AtomicUsize = AtomicUsize::new(0);
static SLEEP_ID: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    pub(crate) static RUNTIME: Rc<Runtime> = {
        Rc::new(Runtime {
            timers: Default::default(),
            timer_queue: Default::default(),
            tasks: Default::default(),
            poll_again: RefCell::new(Default::default()),
        })
    };
}

pub fn auto_yield() -> Yield {
    Yield(unsafe { crate::ffi::yield_rt } != 0)
}

// Single threaded runtime
pub struct Runtime {
    timers: RefCell<BTreeMap<(Instant, usize), Waker>>,
    timer_queue: RefCell<VecDeque<TimerOp>>,

    tasks: RefCell<HashMap<usize, Rc<TaskHandle>>>,
    poll_again: RefCell<VecDeque<usize>>,
}

pub struct JoinHandle<T: 'static> {
    _phantom: PhantomData<T>,
    handle: Rc<TaskHandle>,
}

type DynFuture = Pin<Box<dyn Future<Output = Box<dyn Any + 'static>> + 'static>>;
struct TaskHandle {
    future: RefCell<DynFuture>,
    result: RefCell<Option<Box<dyn Any + 'static>>>,
    join_waker: RefCell<Option<Waker>>,
}

struct TaskWaker {
    task_id: usize,
}

enum TimerOp {
    Insert(Instant, usize, Waker),
    Remove(Instant, usize),
}

pub struct SleepHandle(Instant, usize);

impl Runtime {
    /// returns a duration in microseconds till the next poll is required
    pub(crate) fn poll(&self) -> u64 {
        IN_RUNTIME.store(true, Ordering::Release);

        let mut wakers = Vec::<Waker>::new();

        let next_timer_wakeup = self.process_timers(&mut wakers);

        for waker in wakers {
            if let Err(_err) = std::panic::catch_unwind(|| waker.wake()) {
                // TODO: Notify of panic
                self.shutdown();
            }
        }

        loop {
            let mut queue = self.poll_again.borrow_mut();

            let next = if let Some(next) = queue.pop_front() {
                next
            } else {
                break;
            };

            // release queue so other wakeups can run
            drop(queue);

            let task = if let Some(task) = self.tasks.borrow_mut().get(&next).cloned() {
                task
            } else {
                // Task seems to be missing, possible race condition; just ignore it
                continue;
            };

            let mut future = task.future.borrow_mut();

            let task_waker = Arc::new(TaskWaker { task_id: next });
            let waker = Waker::from(task_waker);
            let mut ctx = Context::from_waker(&waker);

            let future = (*future).as_mut();

            // FIXME: Add catch unwind
            match future.poll(&mut ctx) {
                Poll::Ready(result) => {
                    self.tasks.borrow_mut().remove(&next);

                    // notify JoinHandle of result
                    task.result.borrow_mut().replace(result);
                    if let Some(waker) = task.join_waker.borrow_mut().take() {
                        waker.wake();
                    }
                }
                Poll::Pending => {
                    if unsafe { crate::ffi::yield_rt } != 0 {
                        // yield from runtime
                        break;
                    }
                }
            }
        }

        IN_RUNTIME.store(false, Ordering::Release);

        if self.tasks.borrow().is_empty() {
            self.shutdown();
        }

        let poll_again =
            !self.timer_queue.borrow().is_empty() || !self.poll_again.borrow().is_empty();

        next_timer_wakeup
            .map(|dur| dur.as_micros() as u64)
            .unwrap_or(if poll_again { 0 } else { u64::MAX })
    }

    pub fn shutdown(&self) -> ! {
        // drop all tasks for clean exit
        for _ in self.tasks.borrow_mut().drain() {}
        unsafe {
            ffi::shutdown_rt();
        }
    }

    pub fn spawn<R: 'static>(&self, future: impl Future<Output = R> + 'static) -> JoinHandle<R> {
        let task = Box::pin(async move {
            let result = future.await;
            Box::new(result) as Box<dyn Any>
        });

        let mut id = TASK_ID.fetch_add(1, Ordering::Relaxed);
        while self.tasks.borrow().contains_key(&id) {
            id = TASK_ID.fetch_add(1, Ordering::Relaxed);
        }

        let task_handle = Rc::new(TaskHandle {
            future: RefCell::new(task),
            result: RefCell::new(None),
            join_waker: RefCell::new(None),
        });
        let join_handle = JoinHandle {
            _phantom: PhantomData,
            handle: task_handle.clone(),
        };

        self.tasks.borrow_mut().insert(id, task_handle);
        self.poll_again.borrow_mut().push_back(id);

        join_handle
    }

    pub fn schedule_sleep(&self, until: Instant, waker: Waker) -> SleepHandle {
        let id = SLEEP_ID.fetch_add(1, Ordering::Relaxed);
        let op = TimerOp::Insert(until, id, waker);
        self.timer_queue.borrow_mut().push_back(op);
        SleepHandle(until, id)
    }

    fn wake(&self) {
        if IN_RUNTIME.load(Ordering::Relaxed) {
            return;
        }
        unsafe {
            ffi::wake();
        }
    }

    // This code is partially taken from https://github.com/smol-rs/async-io/blob/master/src/reactor.rs under the MIT licence
    fn process_timers(&self, wakers: &mut Vec<Waker>) -> Option<Duration> {
        self.process_timer_ops();

        let now = Instant::now();

        // Split timers into ready and pending timers
        // We split exactly after now, so now is also considered ready
        let pending = self
            .timers
            .borrow_mut()
            .split_off(&(now + Duration::from_nanos(1), 0));
        let ready = mem::replace(&mut *self.timers.borrow_mut(), pending);

        let dur = if ready.is_empty() {
            self.timers
                .borrow_mut()
                .keys()
                .next()
                .map(|(when, _)| when.saturating_duration_since(now))
        } else {
            None
        };

        wakers.reserve(ready.len());
        for (_, waker) in ready {
            wakers.push(waker);
        }

        dur
    }

    fn process_timer_ops(&self) {
        for op in self.timer_queue.borrow_mut().drain(..) {
            match op {
                TimerOp::Insert(instant, id, waker) => {
                    self.timers.borrow_mut().insert((instant, id), waker);
                }
                TimerOp::Remove(instant, id) => {
                    let _ = self.timers.borrow_mut().remove(&(instant, id));
                }
            }
        }
    }
}

impl<T: 'static> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.handle.result.borrow_mut().take() {
            // it's fine to unwrap here, since we know the type must be `T`
            let t: Box<T> = result.downcast().unwrap();
            Poll::Ready(*t)
        } else {
            self.handle.join_waker.borrow_mut().replace(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        RUNTIME.with(|rt| {
            rt.poll_again.borrow_mut().push_back(self.task_id);
            rt.wake();
        });
    }
}

impl Drop for SleepHandle {
    fn drop(&mut self) {
        RUNTIME.with(|rt| {
            rt.timer_queue
                .borrow_mut()
                .push_back(TimerOp::Remove(self.0, self.1));
        });
    }
}
