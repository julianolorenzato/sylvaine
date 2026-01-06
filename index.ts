const wasmFile = await Deno.open("./runtime.wasm")

const { instance } = await WebAssembly.instantiateStreaming(wasmFile.readable, {
    console: {
        log: (msg: number) => console.log(`Wasm diz: ${msg}`)
    }
})