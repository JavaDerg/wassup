use std::any::Any;
use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::time::{Duration, Instant};

static IN_RUNTIME: AtomicBool = AtomicBool::new(false);
static RUNTIME: Runtime = Runtime {
    timers: Default::default(),
    timer_queue: Default::default(),
    tasks: Default::default(),
    poll_again: RefCell::new(Default::default()),
};

// Single threaded runtime
struct Runtime {
    timers: RefCell<BTreeMap<(Instant, usize), Waker>>,
    timer_queue: RefCell<VecDeque<TimerOp>>,

    tasks: RefCell<HashMap<usize, Rc<TaskHandle>>>,
    poll_again: RefCell<VecDeque<usize>>,
}

pub struct JoinHandle<T> {
    handle: Rc<TaskHandle>,
}

struct TaskHandle {
    future: RefCell<Pin<Box<dyn Future<Output = Box<dyn Any + 'static>> + Unpin + 'static>>>,
    last_poll: RefCell<Instant>,
    result: RefCell<Option<Box<dyn Any>>>,
    join_waker: RefCell<Option<Waker>>,
}

struct TaskWaker {
    task_id: usize,
}

enum TimerOp {
    Insert(Instant, usize, Waker),
    Remove(Instant, usize),
}

impl Runtime {
    fn poll(&self) {
        IN_RUNTIME.store(true, Ordering::Release);

        let mut wakers = Vec::<Waker>::new();

        let next_timer_wakeup = self.process_timers(&mut wakers);

        for waker in wakers {
            if let Err(_err) = std::panic::catch_unwind(|| waker.wake()) {
                // TODO: Notify of panic
                // FIXME: Do clean exit
            }
        }

        while !self.poll_again.borrow().is_empty() {
            let mut queue = self.poll_again.borrow_mut();
            let next = queue.pop_front().unwrap();
            // release queue so other wakeups can run
            drop(queue);

            let task = self.tasks.borrow_mut().remove(&next).unwrap();
            let mut future = task.future.borrow_mut();

            let task_waker = Arc::new(TaskWaker { task_id: next });
            let waker = Waker::from(task_waker);
            let mut ctx = Context::from_waker(&waker);

            let future = (*future).as_mut();
            match future.poll(&mut ctx) {
                Poll::Ready(result) => {
                    task.result.replace(Some(result));
                    task.join_waker.borrow_mut().take().unwrap().wake();
                }
                Poll::Pending => {}
            }
        }

        IN_RUNTIME.store(false, Ordering::Release);
    }

    fn wake(&self) {
        if IN_RUNTIME.load(Ordering::Relaxed) {
            return;
        }
        // TODO: Notify host to poll again
        todo!()
    }

    // This code is partially taken from https://github.com/smol-rs/async-io/blob/master/src/reactor.rs under the MIT licence
    fn process_timers(&self, wakers: &mut Vec<Waker>) -> Option<Duration> {
        self.process_timer_ops();

        let now = Instant::now();

        // Split timers into ready and pending timers
        // We split exactly after now, so now is also considered ready
        let pending = self.timers.borrow_mut().split_off(&(now + Duration::from_nanos(1), 0));
        let ready = mem::replace(&mut *self.timers.borrow_mut(), pending);

        let dur = if ready.is_empty() {
            self.timers
                .keys()
                .next()
                .map(|(when, )| when.saturating_duration_since(now))
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

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref(&self)
    }

    fn wake_by_ref(self: &Arc<Self>) {
        RUNTIME.poll_again.borrow_mut().push_back(self.task_id);
        RUNTIME.wake();
    }
}
