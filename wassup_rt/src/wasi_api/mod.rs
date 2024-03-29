use wasmer::Function;
use wasmer::{imports, ImportObject, Store};

mod env;
mod syscalls;
mod unix;
mod state;
mod ipc;

pub use env::WasiEnv;
pub use state::State;

pub fn generate_imports(store: &Store, env: WasiEnv) -> ImportObject {
    imports! {
        "wasi_snapshot_preview1" => {
            "sched_yield" => Function::new_native_with_env(store, env.clone(), syscalls::sched_yield),
            "args_get" => Function::new_native_with_env(store, env.clone(), syscalls::args_get),
            "args_sizes_get" => Function::new_native_with_env(store, env.clone(), syscalls::args_sizes_get),
            "clock_res_get" => Function::new_native_with_env(store, env.clone(), syscalls::clock_res_get),
            "clock_time_get" => Function::new_native_with_env(store, env.clone(), syscalls::clock_time_get),
            "fd_write" => Function::new_native_with_env(store, env.clone(), syscalls::fd_write),
            "random_get" => Function::new_native_with_env(store, env.clone(), syscalls::random_get),
            "environ_get" => Function::new_native_with_env(store, env.clone(), syscalls::environ_get),
            "environ_sizes_get" => Function::new_native_with_env(store, env.clone(), syscalls::environ_sizes_get),
            "proc_exit" => Function::new_native_with_env(store, env.clone(), syscalls::proc_exit),
        }
    }
}
