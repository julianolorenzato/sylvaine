mod codegen;
mod parser;
mod semantic_analysis;
mod normalization;
mod lowering;
mod type_checking;

fn main() {
    let code = "
        (define pi (lambda () 3))

        6
    ";

    let code2 = "
    (define left (lambda (a b) b))

    (define right (lambda (a b) a))

    (let ((x (left 3 6)) (y 3)) (right x y))
    ";

    match parser::parse(code.into()) {
        Ok(ast) => {
            println!("{:#?}", ast);
            let ast = normalization::normalize(ast);
            let lower_ast = lowering::lower(ast);
            // let (typed_ast, env) = type_checking::check(lower_ast);

            // println!("{:#?}", ast);
            // println!("{:#?}", typed_ast);
            // println!("{:#?}", env);
            // let typed_ast = type_checking::check(lower_ast);

            let wasm_code = codegen::codegen(&lower_ast);
        }
        Err(_) => (),
    }
}
