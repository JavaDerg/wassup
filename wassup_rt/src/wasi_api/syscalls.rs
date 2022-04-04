use rand::RngCore;
use wasi::{Errno, ERRNO_ADDRNOTAVAIL, ERRNO_SUCCESS};
use wasmer::{Array, WasmPtr};
use wasmer_types::ValueType;
use crate::wasi_api::env::WasiEnv;
use crate::wasi_api::unix::{platform_clock_res_get, platform_clock_time_get};

macro_rules! deref_item(
    ($arg:expr, $memory:expr) => {
        if let Some(cell) = $arg.deref($memory) {
            cell
        } else {
            return ERRNO_ADDRNOTAVAIL;
        }
    }
);

pub fn args_get(
    _env: &WasiEnv,
    _argv: WasmPtr<WasmPtr<u8, Array>, Array>,
    _argv_buf: WasmPtr<u8, Array>,
) -> Errno {
    ERRNO_SUCCESS
}

pub fn args_sizes_get(
    env: &WasiEnv,
    argc: WasmPtr<u32>,
    argv_buf_size: WasmPtr<u32>,
) -> Errno {
    let memory = env.memory();

    deref_item!(argc, memory).set(0);
    deref_item!(argv_buf_size, memory).set(0);

    ERRNO_SUCCESS
}

pub fn sched_yield(
    env: &WasiEnv,
) -> Errno {
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

pub fn fd_write(
    env: &WasiEnv,
    fd: wasi::Fd,
    iovs: WasmPtr<Ciovec, Array>,
    iovs_len: u32,
    nwritten: WasmPtr<u32>,
) -> Errno {
    todo!()
}

pub fn random_get(
    env: &WasiEnv,
    buf: WasmPtr<u8, Array>,
    buf_len: u32,
) -> Errno {
    let memory = env.memory();

    let mut rand_buf = vec![0; buf_len as usize];
    rand::thread_rng().fill_bytes(&mut rand_buf);

    unsafe {
        memory.view::<u8>()
            .subarray(buf.offset(), (buf.offset() + buf_len))
            .copy_from(&rand_buf);
    }

    ERRNO_SUCCESS
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Ciovec {
    ptr: WasmPtr<u8, Array>,
    len: wasi::Size,
}

unsafe impl ValueType for Ciovec {}
