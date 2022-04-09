use crate::wasi_api::env::WasiEnv;
use crate::wasi_api::unix::{platform_clock_res_get, platform_clock_time_get};

use rand::RngCore;
use std::io::Write;
use wasi::{Errno, ERRNO_ADDRNOTAVAIL, ERRNO_BADF, ERRNO_INVAL, ERRNO_IO, ERRNO_SUCCESS};

use wasmer::{Array, WasmError, WasmPtr};
use wasmer_types::ValueType;

macro_rules! deref_item (
    ($arg:expr, $memory:expr) => {
        if let Some(cell) = $arg.deref($memory) {
            cell
        } else {
            return ERRNO_ADDRNOTAVAIL;
        }
    };
);

macro_rules! deref_array (
    ($arg:expr, $start:expr => $end:expr, $memory:expr) => {
        if let Some(cell) = $arg.deref($memory, $start, $end) {
            cell
        } else {
            return ERRNO_ADDRNOTAVAIL;
        }
    };
);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Ciovec {
    ptr: WasmPtr<u8, Array>,
    len: u32,
}

unsafe impl ValueType for Ciovec {}

pub fn args_get(
    _env: &WasiEnv,
    _argv: WasmPtr<WasmPtr<u8, Array>, Array>,
    _argv_buf: WasmPtr<u8, Array>,
) -> Errno {
    ERRNO_SUCCESS
}

pub fn args_sizes_get(env: &WasiEnv, argc: WasmPtr<u32>, argv_buf_size: WasmPtr<u32>) -> Errno {
    let memory = env.memory();

    deref_item!(argc, memory).set(0);
    deref_item!(argv_buf_size, memory).set(0);

    ERRNO_SUCCESS
}

pub fn sched_yield(_env: &WasiEnv) -> Errno {
    ERRNO_SUCCESS
}

pub fn clock_res_get(
    env: &WasiEnv,
    clock_id: wasi::Clockid,
    resolution: WasmPtr<wasi::Timestamp>,
) -> Errno {
    let memory = env.memory();

    let out_addr = deref_item!(resolution, memory);

    platform_clock_res_get(clock_id, out_addr)
}

pub fn clock_time_get(
    env: &WasiEnv,
    clock_id: wasi::Clockid,
    precision: wasi::Timestamp,
    time: WasmPtr<wasi::Timestamp>,
) -> Errno {
    let memory = env.memory();

    let out_addr = deref_item!(time, memory);

    platform_clock_time_get(clock_id, precision, out_addr)
}

pub fn random_get(env: &WasiEnv, buf: WasmPtr<u8, Array>, buf_len: u32) -> Errno {
    let memory = env.memory();

    let mut rand_buf = vec![0; buf_len as usize];
    rand::thread_rng().fill_bytes(&mut rand_buf);

    let view = memory.view::<u8>();

    let end = buf.offset() + buf_len;
    if view.len() < end as usize {
        return ERRNO_INVAL;
    }

    unsafe {
        view.subarray(buf.offset(), end).copy_from(&rand_buf);
    }

    ERRNO_SUCCESS
}

pub fn environ_get(
    _env: &WasiEnv,
    _environ: WasmPtr<WasmPtr<u8, Array>, Array>,
    _environ_buf: WasmPtr<u8, Array>,
) -> Errno {
    ERRNO_SUCCESS
}

pub fn environ_sizes_get(
    env: &WasiEnv,
    environ_count: WasmPtr<u32>,
    environ_buf_size: WasmPtr<u32>,
) -> Errno {
    let memory = env.memory();

    deref_item!(environ_count, memory).set(0);
    deref_item!(environ_buf_size, memory).set(0);

    ERRNO_SUCCESS
}

pub fn proc_exit(_env: &WasiEnv, _error_code: wasi::Exitcode) -> Result<(), WasmError> {
    Err(WasmError::Generic("proc_exit".to_string()))
}

pub fn fd_write(
    env: &WasiEnv,
    fd: wasi::Fd,
    iovs: WasmPtr<Ciovec, Array>,
    iovs_len: u32,
    nwritten: WasmPtr<u32>,
) -> Errno {
    let mut stream: Box<dyn Write> = match fd {
        1 => Box::new(std::io::stdout()),
        2 => Box::new(std::io::stderr()),
        _ => return ERRNO_BADF,
    };

    let memory = env.memory();
    let iovs = deref_array!(iovs, 0 => iovs_len, memory);
    let mut written = 0;
    for iov in iovs {
        let Ciovec { ptr, len }: Ciovec = iov.get();

        let end = ptr.offset() + len as u32;

        let mut view = memory.view::<u8>();
        if view.len() < end as usize {
            return ERRNO_INVAL;
        }
        view = view.subarray(ptr.offset(), end);
        let slice = &view[..];

        // SAFETY: we have exclusive access to the memory and Cell is transparent type
        let slice_u8: &[u8] = unsafe { std::mem::transmute(slice) };

        if let Err(_) = stream.write_all(slice_u8) {
            return ERRNO_IO;
        };

        written += len;
    }

    deref_item!(nwritten, memory).set(written as u32);

    ERRNO_SUCCESS
}
