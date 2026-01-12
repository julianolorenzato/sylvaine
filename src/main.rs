use std::{fs::File, io::Write};

mod codegen;
mod semantic_analysis;
mod syntax_analysis;

fn main() {
    let code = "
        (define pi (lambda () 3))
    ";

    let code2 = "
    (define left (lambda (a b) b))

    (define right (lambda (a b) a))

    (let ((x (left 3 6)) (y 3)) (right x y))
    ";

    let code3 = "(define sum (+ 4 5))";

    let ast = syntax_analysis::parse(code3.into());
    let wasm_code = codegen::codegen(&ast);

    File::create("./wasm/build/code.wasm")
        .unwrap()
        .write(&wasm_code)
        .unwrap();
}
