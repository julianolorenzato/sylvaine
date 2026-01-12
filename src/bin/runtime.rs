use wasmtime::{AnyRef, Config, Engine, Error, Linker, Module, Rooted, Store};

fn main() -> Result<(), Error> {
    let mut config = Config::new();

    config.wasm_gc(true);
    config.wasm_reference_types(true);
    config.wasm_function_references(true);

    let engine = Engine::new(&config)?;

    let mut store = Store::new(&engine, ());

    let module = Module::from_file(&engine, "wasm/build/code.wasm")?;

    let linker = Linker::new(&engine);

    let instance = linker.instantiate(&mut store, &module)?;

    let main_func = instance.get_typed_func::<(), Option<Rooted<AnyRef>>>(&mut store, "main")?;

    let result = main_func.call(&mut store, ())?;

    println!("Result: {:?}", result);

    Ok(())
}
