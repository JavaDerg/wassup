use std::collections::VecDeque;
use std::sync::Arc;
use bytes::Bytes;
use crossbeam_queue::{ArrayQueue};
use wasi::Errno;

pub struct Ipc(Arc<InnerIpc>);
struct InnerIpc {
    id: u32,
    recv_buff: ArrayQueue<Bytes>,
}

pub mod syscalls {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use crossbeam_queue::ArrayQueue;
    use wasi::{Errno, ERRNO_INVAL, ERRNO_NOBUFS, ERRNO_NODEV, ERRNO_SUCCESS};
    use wasmer_types::ValueType;
    use crate::wasi_api::ipc::{InnerIpc, Ipc};
    use crate::WasiEnv;

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct IpcMakeChannelResult {
        id: u32,
        err: Errno,
    }

    unsafe impl ValueType for IpcMakeChannelResult {}

    pub fn ipc_make_channel(env: &WasiEnv) -> IpcMakeChannelResult {
        let state = &*env.state;

        if state.ipcs.len() >= 128 {
            return IpcMakeChannelResult {
                id: 0,
                err: ERRNO_NOBUFS,
            };
        }

        // This is technically a race condition, but I find it incredibly unlikely that for it to occur
        let mut step = 1;
        let id = loop {
            let next_id = state.next_id.fetch_add(step, Ordering::Relaxed);
            if !state.ipcs.contains_key(&next_id) {
                break next_id;
            }
            // make steps larger each time to increase chance of hitting a free one
            step = if let Ok(n) = step.checked_add(step) {
                n
            } else {
                return IpcMakeChannelResult {
                    id: 0,
                    err: ERRNO_NOBUFS,
                };
            };
        };
        let mut ipc = Ipc(Arc::new(InnerIpc {
            id,
            recv_buff: ArrayQueue::new(128),
        }));
        state.ipcs.insert(id, ipc);
        
        IpcMakeChannelResult {
            id,
            err: 0,
        }
    }

    pub fn ipc_drop_channel(env: &WasiEnv, id: u32) -> Errno {
        if env.state.ipcs.remove(&id).is_some() {
            ERRNO_SUCCESS
        } else {
            ERRNO_INVAL
        }
    }

    pub fn ipc_send_msg(buffer: *const u8, len_buf: usize) -> Errno {
        todo!()
    }

    pub fn ipc_recv_msg(buffer: *mut u8, len_buf: usize) -> Errno {
        todo!()
    }
}
