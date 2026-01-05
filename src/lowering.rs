use crate::parser::Expr as GenericExpr;

#[derive(Debug, Clone)]
pub enum Expr {
    Prog(Vec<Expr>),
    Integer(i32),
    Float(f64),
    Call(String, Vec<Expr>),
    Define(String, Box<Expr>),
    Lambda(Vec<String>, Box<Expr>),
    Let(Vec<(String, Expr)>, Box<Expr>),
    Symbol(String),
}

pub fn lower(ast: GenericExpr) -> Expr {
    match ast {
        GenericExpr::Prog(exprs) => {
            let mut prog_exprs = vec![];
            for expr in exprs {
                prog_exprs.push(lower(expr));
            }

            Expr::Prog(prog_exprs)
        }
        GenericExpr::Symbol(s) => Expr::Symbol(s),
        GenericExpr::Integer(n) => Expr::Integer(n),
        GenericExpr::Float(n) => Expr::Float(n),
        GenericExpr::Nil => todo!(),
        GenericExpr::List(xs) => {
            let mut it = xs.into_iter();

            // Match first list item
            match it.next() {
                Some(GenericExpr::Symbol(s)) if s == "define" => {
                    let name = match it.next() {
                        Some(GenericExpr::Symbol(n)) => n,
                        None => panic!("expected name"),
                        _ => panic!("name must be a symbol"),
                    };

                    let expr = it.next().expect("expected expr");

                    Expr::Define(name, Box::new(lower(expr)))
                }
                Some(GenericExpr::Symbol(s)) if s == "lambda" => {
                    let params = match it.next() {
                        Some(GenericExpr::List(params)) => params,
                        None => panic!("expected lambda params"),
                        _ => panic!("lambda params must be a list"),
                    };

                    let body = it.next().expect("expected lambda body");

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
                Some(GenericExpr::Symbol(s)) if s == "let" => {
                    let bindings = match it.next() {
                        Some(GenericExpr::List(bindings)) => bindings,
                        None => panic!("expected let bindings"),
                        _ => panic!("let bindings must be a list"),
                    };

                    let body = it.next().expect("expected let body");

                    let mut let_bindings = vec![];
                    for (i, binding) in bindings.into_iter().enumerate() {
                        if let GenericExpr::List(mut pair) = binding {
                            let value = pair.pop().expect("let binding {i} must have two elements");
                            let key = pair.pop().expect("let binding {i} must have two elements");

                            if let GenericExpr::Symbol(name) = key {
                                let_bindings.push((name, lower(value)));
                            } else {
                                panic!("let binding {i} must have a symbol as first element")
                            }
                        } else {
                            panic!("let binding {i} must be a list")
                        }
                    }

                    Expr::Let(let_bindings, Box::new(lower(body)))
                }
                Some(GenericExpr::Symbol(name)) => {
                    let args: Vec<GenericExpr> = it.collect();

                    let mut call_args = vec![];

                    for arg in args {
                        call_args.push(lower(arg));
                    }

                    Expr::Call(name, call_args)
                }
                Some(_) => panic!("invalid construction"),
                None => panic!("empty list"),
            }
        }
    }
}

// match xs.as_slice() {
// // Define
// [GenericExpr::Symbol(s), GenericExpr::Symbol(name), expr] if s == "define" => {
//     Expr::Define(name.to_string(), Box::new(lower(expr)))
// }
// // Lambda
// [GenericExpr::Symbol(s), GenericExpr::List(params), body] if s == "lambda" => {
//     let mut lambda_params = vec![];
//     for param in params {
//         if let GenericExpr::Symbol(name) = param {
//             lambda_params.push(name.to_string());
//         } else {
//             panic!("all lambda params must be strings");
//         }
//     }

//     Expr::Lambda(lambda_params, Box::new(lower(*body)))
// }
// // Let
// [GenericExpr::Symbol(s), GenericExpr::List(bindings), body] if s == "let" => {
//     let mut let_bindings = vec![];
//     for binding in bindings {
//         if let GenericExpr::List(elements) = binding {
//             match elements.as_slice() {
//                 [GenericExpr::Symbol(name), expr] => {
//                     let_bindings.push((name.to_string(), lower(*expr)));
//                 }
//                 _ => panic!("each let binding must be a pair of name/expression"),
//             }
//         } else {
//             panic!("the let argument must be a list")
//         }
//     }

//     Expr::Let(let_bindings, Box::new(lower(*body)))
// }
// // Function Call
// [GenericExpr::Symbol(s), args @ ..] => {
//     let mut call_args = vec![];
//     for arg in args {
//         call_args.push(lower(*arg));
//     }

//     Expr::Call(s.to_string(), call_args)
// }
// _ => panic!("invalid construction"),
