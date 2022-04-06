use wasmer::{LazyInit, Memory, WasmerEnv};

#[derive(Clone, WasmerEnv)]
pub struct WasiEnv {
    #[wasmer(export)]
    pub(crate) memory: LazyInit<Memory>,
}

impl WasiEnv {
    pub fn memory(&self) -> &Memory {
        self.memory_ref().unwrap()
    }
}
