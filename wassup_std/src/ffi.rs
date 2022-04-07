use std::sync::atomic::Ordering;
use bytes::{Bytes, BytesMut};
use wasi::{Errno, ERRNO_NOENT, ERRNO_NXIO, ERRNO_SUCCESS};
use crate::ipc::IPCS;
use crate::runtime::RUNTIME;

type Duration = u64;

extern "C" {
    // async runtime interface
    pub static yield_rt: u32;
    pub fn wake();
    pub fn shutdown_rt() -> !;

    // ipc interface
    /// returns:
    ///  .0: channel id
    ///  .1: buffer size
    ///  .2: error
    pub fn ipc_make_channel() -> IpcMakeChannelResult;
    pub fn ipc_drop_channel(id: u32) -> Errno;
    pub fn ipc_send_msg(buffer: *const u8, len_buf: usize) -> Errno;
    pub fn ipc_recv_msg(buffer: *mut u8, len_buf: usize) -> Errno;
}

#[repr(C)]
pub struct IpcSegment {
    pub offset: usize,
    pub len: usize,
}

#[repr(C)]
pub struct IpcMakeChannelResult {
    pub id: u32,
    pub size: u32,
    pub err: Errno,
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

#[no_mangle]
pub extern "C" fn ipc_notify(id: u32, seg_size: u32) -> Errno {
    IPCS.with(|ipcs| {
        let mut map = ipcs.borrow_mut();

        let rc = map
            .get(&id)
            .map(|recv| recv.upgrade())
            .flatten();
        if let Some(rc) = rc {
            let mut receiver = (*rc).borrow_mut();

            let mut buf = vec![0u8; seg_size as usize];
            let err = unsafe { ipc_recv_msg(buf.as_mut_ptr(), buf.len()) };
            if err != ERRNO_SUCCESS {
                return err;
            }
            let bytes = Bytes::from(buf);
            receiver.buffer.push_back(bytes);

            if let Some(waker) = &receiver.waker {
                receiver.flag.store(true, Ordering::Release);
                waker.wake_by_ref();
            }

            return ERRNO_SUCCESS;
        }

        ERRNO_NXIO
    })
}