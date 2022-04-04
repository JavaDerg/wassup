use libc::{clock_getres, clock_gettime, CLOCK_MONOTONIC, CLOCK_PROCESS_CPUTIME_ID, CLOCK_REALTIME, CLOCK_THREAD_CPUTIME_ID, timespec};
use wasmer::WasmCell;

///! this file is partially taken from https://github.com/wasmerio/wasmer/blob/3604debecc753c1a13701559035d75b30b77d912/lib/wasi/src/syscalls/unix/mod.rs and under MIT licence

pub fn platform_clock_res_get(
    clock_id: wasi::Clockid,
    resolution: WasmCell<wasi::Timestamp>,
) -> wasi::Errno {
    let unix_clock_id = match clock_id {
        wasi::CLOCKID_MONOTONIC => CLOCK_MONOTONIC,
        wasi::CLOCKID_PROCESS_CPUTIME_ID => CLOCK_PROCESS_CPUTIME_ID,
        wasi::CLOCKID_REALTIME => CLOCK_REALTIME,
        wasi::CLOCKID_THREAD_CPUTIME_ID => CLOCK_THREAD_CPUTIME_ID,
        _ => return wasi::ERRNO_INVAL,
    };

    let (output, timespec_out) = unsafe {
        let mut timespec_out: timespec = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        (clock_getres(unix_clock_id, &mut timespec_out), timespec_out)
    };

    let t_out = (timespec_out.tv_sec * 1_000_000_000).wrapping_add(timespec_out.tv_nsec);
    resolution.set(t_out as wasi::Timestamp);

    match output {
        0 => wasi::ERRNO_SUCCESS,
        libc::EACCES => wasi::ERRNO_ACCES,
        libc::EFAULT => wasi::ERRNO_FAULT,
        libc::EINVAL => wasi::ERRNO_INVAL,
        libc::ENODEV => wasi::ERRNO_NODEV,
        libc::ENOTSUP => wasi::ERRNO_NOTSUP,
        libc::EPERM => wasi::ERRNO_PERM,
        _ => unreachable!("undefined error code"),
    }
}

pub fn platform_clock_time_get(
    clock_id: wasi::Clockid,
    _precision: wasi::Timestamp,
    time: WasmCell<wasi::Timestamp>,
) -> wasi::Errno {
    let unix_clock_id = match clock_id {
        wasi::CLOCKID_MONOTONIC => CLOCK_MONOTONIC,
        wasi::CLOCKID_PROCESS_CPUTIME_ID => CLOCK_PROCESS_CPUTIME_ID,
        wasi::CLOCKID_REALTIME => CLOCK_REALTIME,
        wasi::CLOCKID_THREAD_CPUTIME_ID => CLOCK_THREAD_CPUTIME_ID,
        _ => return wasi::ERRNO_INVAL,
    };

    let (output, timespec_out) = unsafe {
        let mut timespec_out: timespec = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        (clock_gettime(unix_clock_id, &mut timespec_out), timespec_out)
    };

    let t_out = (timespec_out.tv_sec * 1_000_000_000).wrapping_add(timespec_out.tv_nsec);
    time.set(t_out as wasi::Timestamp);

    match output {
        0 => wasi::ERRNO_SUCCESS,
        libc::EACCES => wasi::ERRNO_ACCES,
        libc::EFAULT => wasi::ERRNO_FAULT,
        libc::EINVAL => wasi::ERRNO_INVAL,
        libc::ENODEV => wasi::ERRNO_NODEV,
        libc::ENOTSUP => wasi::ERRNO_NOTSUP,
        libc::EPERM => wasi::ERRNO_PERM,
        _ => unreachable!("undefined error code"),
    }
}
