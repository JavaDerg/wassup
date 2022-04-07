use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::{Future, Ready};
use std::io::Write;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};
use bytes::{Bytes, BytesMut};
use wasi::ERRNO_SUCCESS;
use crate::ffi;
use crate::ffi::{IpcMakeChannelResult, IpcSegment};

type RefMap<K, V> = RefCell<HashMap<K, V>>;
type SharedIpcReceiver = Rc<RefCell<IpcReceiver>>;
type WeaklySharedIpcReceiver = Weak<RefCell<IpcReceiver>>;

thread_local! {
    pub(crate) static IPCS: RefMap<u32, WeaklySharedIpcReceiver> =
        RefCell::new(HashMap::new());
}

pub struct Ipc {
    id: u32,
    receiver: SharedIpcReceiver,
}

pub(crate) struct IpcReceiver {
    pub buffer: VecDeque<Bytes>,
    pub waker: Option<Waker>,
    pub flag: Rc<AtomicBool>,
    pub dropped: bool,
}

struct Recv(Rc<RefCell<IpcReceiver>>);

impl Ipc {
    pub fn new() -> Self {
        let IpcMakeChannelResult {
            id, size, err
        } = unsafe { ffi::ipc_make_channel() };
        if err != wasi::ERRNO_SUCCESS {
            panic!("ipc_make_channel failed: {} ({}): {}", wasi::errno_name(err), err, wasi::errno_docs(err));
        }

        let receiver = Rc::new(
            RefCell::new(
                IpcReceiver {
                    buffer: VecDeque::with_capacity(size as usize),
                    waker: None,
                    flag: Rc::new(Default::default()),
                    dropped: true,
                },
            ),
        );

        IPCS.with(|ipcs| {
            ipcs.borrow_mut().insert(id, Rc::downgrade(&receiver));
        });

        Self {
            id,
            receiver,
        }
    }

    pub fn send(&mut self, msg: Bytes) {
        assert_eq!(
            unsafe { ffi::ipc_send_msg(
                msg.as_ptr(),
                msg.len(),
            ) },
            ERRNO_SUCCESS,
        );
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        if let Some(msg) = self.try_recv() {
            return Some(msg);
        } else {
            Recv(self.receiver.clone()).await;
            self.try_recv()
        }
    }

    pub fn try_recv(&mut self) -> Option<Bytes> {
        (*self.receiver).borrow_mut().buffer.pop_front()
    }
}

impl Drop for Ipc {
    fn drop(&mut self) {
        assert_eq!(unsafe { ffi::ipc_drop_channel(self.id) }, wasi::ERRNO_SUCCESS);
    }
}

impl Future for Recv {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut receiver = (*self.0).borrow_mut();
        if receiver.waker.is_none() {
            receiver.waker = Some(cx.waker().clone());
            receiver.flag.store(true, Ordering::Release);
        }

        if receiver.flag.load(Ordering::Acquire) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
