mod codegen;
mod parser;
mod semantic_analysis;
mod normalization;
mod lowering;
mod type_checking;

fn main() {
    let code = "
        (define pi (lambda () 3)))
    ";

    match parser::parse("(define (a b) (quote a b))".into()) {
        Ok(ast) => {
            let ast = normalization::normalize(ast);
            let lower_ast = lowering::lower(&ast);
            let typed_ast = type_checking::check(&lower_ast);

            // let wasm_code = codegen::codegen(&ast);

            // println!("{:?}", wasm_code);
        }
        Err(_) => (),
    }
}
