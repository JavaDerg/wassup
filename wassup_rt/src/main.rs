extern crate core;

use crate::transformer::ModuleTransformer;
use crate::wasi_api::{State, WasiEnv};


use std::fmt::Display;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wasmer::{
    imports, CompilerConfig, Export, Function, Global, Instance, Module,
    Resolver, Store, Value,
};
use wasmer_compiler_llvm::{LLVMOptLevel, LLVM};
use wasmer_engine_universal::Universal;

mod transformer;
mod wasi_api;

struct ComboResolver<'a, const N: usize>(pub [&'a (dyn Resolver + Sync + Send); N]);

impl<'a, const N: usize> Resolver for ComboResolver<'a, N> {
    fn resolve(&self, index: u32, module: &str, field: &str) -> Option<Export> {
        for r in self.0 {
            if let x @ Some(_) = r.resolve(index, module, field) {
                return x;
            }
        }
        None
    }
}

fn main() {
    let mut stamper = Stamper::now();

    let wasm = std::fs::read("target/wasm32-wasi/release/test_bin.wasm").unwrap();
    stamper.stamp("wasm loaded");

    let mut compiler = LLVM::default();
    compiler.opt_level(LLVMOptLevel::Aggressive);
    compiler.push_middleware(Arc::new(ModuleTransformer::default()));
    stamper.stamp("mk-compiler");

    let store = Store::new(&Universal::new(compiler).engine());
    stamper.stamp("mk-store");

    let module = Module::new(&store, wasm).unwrap();
    stamper.stamp("compile module");

    let env_imports = imports! {
        "env" => {
            "yield_rt" => Global::new(&store, Value::I32(0)),
            "wake" => Function::new_native(&store, || println!("wakeup lmao")),
            "log_n" => Function::new_native(&store, |_: u64| ()),
            "shutdown_rt" => Function::new_native(&store, || {
                println!("exit trigger");
                std::process::exit(0)
            }),
        }
    };
    let wasi_imports = wasi_api::generate_imports(
        &store,
        WasiEnv {
            memory: Default::default(),
            state: Arc::new(State::new())
        },
    );

    let imports = ComboResolver([&env_imports, &wasi_imports]);
    stamper.stamp("mk-imports");

    let instance = Instance::new(&module, &imports).unwrap();
    stamper.stamp("mk-instance");

    let start = instance
        .exports
        .get_native_function::<(), ()>("_start")
        .expect("_start must be present");
    let poll = instance
        .exports
        .get_native_function::<(), u64>("poll_runtime")
        .expect("poll_runtime must be present");

    start.call().unwrap();

    loop {
        let sleep_time = poll.call().unwrap();

        std::thread::sleep(Duration::from_micros(sleep_time))
    }
}

struct Stamper(Instant, Instant);

impl Stamper {
    pub fn now() -> Self {
        let now = Instant::now();
        Self(now, now)
    }

    pub fn stamp(&mut self, d: impl Display) {
        let now = Instant::now();
        let last = self.1;
        println!("[{:?} ~ {:?}] {}", now - self.0, now - last, d);
        self.1 = Instant::now();
    }
}
