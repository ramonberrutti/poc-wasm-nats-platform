use std::u64;

use wasmtime::*;
use wasmtime_wasi::{p1::WasiP1Ctx, WasiCtx};

struct State {
    nc: async_nats::Client,
    wasi: WasiP1Ctx,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let nc = async_nats::connect("localhost:4222").await?;

    let mut config = Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config)?;

    // let wat = r#"
    //     (module
    //         (import "nats" "publish"
    //             (func $publish (param i32 i32 i32 i32)))

    //         (memory (export "memory") 1)

    //         (data (i32.const 0) "hello.subject")
    //         (data (i32.const 100) "hello payload")

    //         (func (export "run")
    //             ;; subject ptr=0 len=13
    //             ;; payload ptr=100 len=13
    //             i32.const 0
    //             i32.const 13
    //             i32.const 100
    //             i32.const 13
    //             call $publish
    //         )
    //     )
    // "#;
    // let module = Module::new(&engine, wat)?;
    let module = Module::from_file(&engine, "target/wasm32-wasip1/debug/wasm.wasm")?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p1::add_to_linker_async(&mut linker, |cx: &mut State| &mut cx.wasi)?;
    linker.func_wrap_async(
        "nats",
        "publish",
        |mut caller: Caller<'_, State>,
         (sub_ptr, sub_len, payload_ptr, payload_len): (i32, i32, i32, i32)| {
            Box::new(async move {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .expect("memory not found");

                let data = memory.data(&caller);

                // Read subject
                let subject =
                    std::str::from_utf8(&data[sub_ptr as usize..(sub_ptr + sub_len) as usize])
                        .expect("invalid subject");

                // Read payload
                let payload = &data[payload_ptr as usize..(payload_ptr + payload_len) as usize];

                println!("Publishing to {} -> {:?}", subject, payload);

                // Clone client (cheap, Arc internally) and publish message
                let nc = caller.data().nc.clone();
                nc.publish(subject.to_string(), payload.to_vec().into())
                    .await
                    .unwrap();
            })
        },
    )?;

    let wasi = WasiCtx::builder()
        .inherit_stdio()
        .env("NAME", "MyName")
        .build_p1();
    let mut store = Store::new(
        &engine,
        State {
            nc: nc.clone(),
            wasi: wasi,
        },
    );
    store.set_fuel(u64::MAX)?;
    store.fuel_async_yield_interval(Some(10000))?;

    let instance = linker.instantiate_async(&mut store, &module).await?;
    let run = instance.get_typed_func::<(), ()>(&mut store, "_start")?;

    run.call_async(&mut store, ()).await?;

    Ok(())
}
