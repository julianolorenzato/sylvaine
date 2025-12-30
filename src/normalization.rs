use crate::parser::Expr;

// struct Normalizer {
//     ast: Expr,
// }

// impl Normalizer {
//     fn new(ast: Expr) -> Self {
//         Self { ast }
//     }

//     pub fn transform(&mut self) {
//         self.desugar();
//         self.expand_macros();
//     }

//     fn desugar(&mut self) {}

//     fn expand_macros(&mut self) {}
// }


pub fn normalize(ast: Expr) -> Expr {
    ast
}