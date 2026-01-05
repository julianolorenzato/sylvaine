mod codegen;
mod semantic_analysis;
mod syntax_analysis;

fn main() {
    let code = "
        (define pi (lambda () 3))

        (define ss (a b) (add a b))
        6
    ";

    let code2 = "
    (define left (lambda (a b) b))

    (define right (lambda (a b) a))

    (let ((x (left 3 6)) (y 3)) (right x y))
    ";

    let ast = syntax_analysis::parse(code.into());

    println!("{:#?}", ast);
}
