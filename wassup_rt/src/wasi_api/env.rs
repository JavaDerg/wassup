use std::sync::Arc;
use wasmer::{LazyInit, Memory, WasmerEnv};
use crate::wasi_api::state::State;

#[derive(Clone, WasmerEnv)]
pub struct WasiEnv {
    #[wasmer(export)]
    pub(crate) memory: LazyInit<Memory>,

    pub state: Arc<State>,
}

impl WasiEnv {
    pub fn memory(&self) -> &Memory {
        self.memory_ref().unwrap()
    }
}