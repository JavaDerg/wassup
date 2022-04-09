use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Mutex;
use dashmap::DashMap;
use crate::wasi_api::ipc::Ipc;

pub struct State {
    pub ipcs: DashMap<u32, Ipc>,
    pub next_id: AtomicU32,
}

impl State {
    pub fn new() -> Self {
        Self {
            ipcs: Default::default(),
            next_id: Default::default()
        }
    }
}
