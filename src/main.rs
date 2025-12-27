mod codegen;
mod parser;

fn main() {
    match parser::parse("(define (a b) (quote a b))".into()) {
        Ok(ast) => {
            println!("{:?}", ast);

            let wasm_code = codegen::gen_webassembly_code(ast);

            println!("{:?}", wasm_code);
        }
        Err(_) => (),
    }
}
