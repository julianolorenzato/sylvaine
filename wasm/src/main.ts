import { p } from "@build/runtime.wasm";
const wasmFile = await Deno.open("build/runtime.wasm");

const buffer = new Uint8Array(50000);

wasmFile.read(buffer);

const { instance } = await WebAssembly.instantiate(buffer, {
  
});

console.log({ instance });
