use std::cell::{Cell, RefCell};
use std::fmt::Display;
use std::sync::Arc;
use std::time::Instant;
use criterion::{black_box, Criterion};
use wasmer::{CompilerConfig, Function, imports, Instance, Module, NativeFunc, Store, wat2wasm};
use wasmer_compiler_llvm::{LLVM, LLVMOptLevel};
use wasmer_engine_universal::Universal;
use crate::transformer::ModuleTransformer;

mod transformer;

fn main() {
    let mut stamper = Stamper::now();
    let wasm = wat2wasm(
        br#"
(module
    (func $boop (import "env" "boop"))
    (global $cc (mut i64) (i64.const 0))
    (func $sum_f (param $x i32) (param $y i32) (result i32)
        call $increment
        local.get $x
        local.get $y
        i32.add
    )
    (func $increment
        global.get $cc
        i64.const 1
        i64.add
        global.set $cc
    )
    (func $num_calls (result i64)
        global.get $cc
    )
    (export "sum" (func $sum_f))
    (export "num_calls" (func $num_calls))
)
"#
    ).unwrap();
    stamper.stamp( "wat2wasm");

    let mut compiler = LLVM::default();
    compiler.opt_level(LLVMOptLevel::Aggressive);
    compiler.push_middleware(Arc::new(ModuleTransformer::default()));
    stamper.stamp( "mk-compiler");

    let store = Store::new(&Universal::new(compiler).engine());
    stamper.stamp( "mk-store");

    let module = Module::new(&store, wasm).unwrap();
    stamper.stamp( "compile module");

    let imports = imports! {
        "env" => {
            "boop" => Function::new_native(&store, || {
                println!("Boop!");
            }),
        }
    };
    stamper.stamp("mk-imports");

    let instance = Instance::new(&module, &imports).unwrap();
    stamper.stamp("mk-instance");

    let sum = instance.exports.get_native_function::<(u32, u32), u32>("sum").unwrap();
    let num_calls = instance.exports.get_native_function::<(), u64>("num_calls").unwrap();
    stamper.stamp("get-native-fn");

    for _ in 0..5 {
        let _ = sum.call(0xDEADBEEF, 0x13376969).unwrap();
        stamper.stamp("exec");
    }

    let mut criterion = Criterion::default().with_output_color(true);
    criterion.bench_function("sum", |b| b.iter(|| {
        sum.call(black_box(0xDEADBEEF),black_box(0x13376969))
    }));
    stamper.stamp("benchmark");

    let _ = sum.call(1, 2).unwrap();
    stamper.stamp("exec 2");

    let calls = num_calls.call().unwrap();
    stamper.stamp(calls);
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