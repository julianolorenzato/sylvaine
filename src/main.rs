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

    let code3 = "
        (define sum (lambda (a b) (+ a b)))
        (sum 3 6)
    ";

    let ast = syntax_analysis::parse(code3.into());
    println!("{:?}", ast);
    let wasm_code = codegen::codegen(&ast);

    File::create("./wasm/build/code.wasm")
        .unwrap()
        .write(&wasm_code)
        .unwrap();
}
