mod codegen;
mod parser;

fn main() {
    let code = "
        (define pi (lambda () 3)))
    ";

    match parser::parse("(define (a b) (quote a b))".into()) {
        Ok(ast) => {
            // println!("{:?}", ast);

            let wasm_code = codegen::codegen(&ast);

            // println!("{:?}", wasm_code);
        }
        Err(_) => (),
    }
}
