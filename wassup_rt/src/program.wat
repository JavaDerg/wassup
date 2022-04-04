(module
    (func $sched_yield (import "wasi_snapshot_preview1" "sched_yield") (result i32))
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
    (export "sched_yield" (func $sched_yield))
    (export "num_calls" (func $num_calls))
)
