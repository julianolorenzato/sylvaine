use crate::parser::Expr as GenericExpr;

#[derive(Clone)]
pub enum Expr {
    Integer(i32),
    Float(f64),
    Call(String, Vec<Expr>),
    Define(String, Box<Expr>),
    Lambda(Vec<String>, Box<Expr>),
    Let(Vec<(String, Expr)>, Box<Expr>),
    Symbol(String),
}

pub fn lower(ast: &GenericExpr) -> Expr {
    match ast {
        GenericExpr::Symbol(s) => Expr::Symbol(s.to_string()),
        GenericExpr::Integer(n) => Expr::Integer(*n),
        GenericExpr::Float(n) => Expr::Float(*n),
        GenericExpr::Nil => todo!(),
        GenericExpr::List(xs) => match xs.as_slice() {
            // Define
            [GenericExpr::Symbol(s), GenericExpr::Symbol(name), expr] if s == "define" => {
                Expr::Define(name.to_string(), Box::new(lower(expr)))
            }
            // Lambda
            [GenericExpr::Symbol(s), GenericExpr::List(params), body] if s == "lambda" => {
                let mut lambda_params = vec![];
                for param in params {
                    if let GenericExpr::Symbol(name) = param {
                        lambda_params.push(name.to_string());
                    } else {
                        panic!("all lambda params must be strings");
                    }
                }

                Expr::Lambda(lambda_params, Box::new(lower(body)))
            }
            // Let
            [GenericExpr::Symbol(s), GenericExpr::List(bindings), body] if s == "let" => {
                let mut let_bindings = vec![];
                for binding in bindings {
                    if let GenericExpr::List(elements) = binding {
                        match elements.as_slice() {
                            [GenericExpr::Symbol(name), expr] => {
                                let_bindings.push((name.to_string(), lower(expr)));
                            }
                            _ => panic!("each let binding must be a pair of name/expression"),
                        }
                    } else {
                        panic!("the let argument must be a list")
                    }
                }

                Expr::Let(let_bindings, Box::new(lower(body)))
            }
            // Function Call
            [GenericExpr::Symbol(s), args @ ..] => {
                let mut call_args = vec![];
                for arg in args {
                    call_args.push(lower(arg));
                }

                Expr::Call(s.to_string(), call_args)
            }
            _ => panic!("invalid construction"),
        },
    }
}
