use crate::syntax_analysis::SExpr;

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

pub fn lower(ast: SExpr) -> Expr {
    match ast {
        SExpr::Prog(exprs) => {
            let mut prog_exprs = vec![];
            for expr in exprs {
                prog_exprs.push(lower(expr));
            }

            Expr::Prog(prog_exprs)
        }
        SExpr::Symbol(s) => Expr::Symbol(s),
        SExpr::Integer(n) => Expr::Integer(n),
        SExpr::Float(n) => Expr::Float(n),
        SExpr::Nil => todo!(),
        SExpr::List(xs) => {
            let mut it = xs.into_iter();

            // Match first list item
            match it.next() {
                Some(SExpr::Symbol(s)) if s == "define" => {
                    let name = match it.next() {
                        Some(SExpr::Symbol(n)) => n,
                        None => panic!("expected name"),
                        _ => panic!("name must be a symbol"),
                    };

                    let expr = it.next().expect("expected expr");

                    Expr::Define(name, Box::new(lower(expr)))
                }
                Some(SExpr::Symbol(s)) if s == "lambda" => {
                    let params = match it.next() {
                        Some(SExpr::List(params)) => params,
                        None => panic!("expected lambda params"),
                        _ => panic!("lambda params must be a list"),
                    };

                    let body = it.next().expect("expected lambda body");

                    let mut lambda_params = vec![];
                    for param in params {
                        if let SExpr::Symbol(name) = param {
                            lambda_params.push(name.to_string());
                        } else {
                            panic!("all lambda params must be strings");
                        }
                    }

                    Expr::Lambda(lambda_params, Box::new(lower(body)))
                }
                Some(SExpr::Symbol(s)) if s == "let" => {
                    let bindings = match it.next() {
                        Some(SExpr::List(bindings)) => bindings,
                        None => panic!("expected let bindings"),
                        _ => panic!("let bindings must be a list"),
                    };

                    let body = it.next().expect("expected let body");

                    let mut let_bindings = vec![];
                    for (i, binding) in bindings.into_iter().enumerate() {
                        if let SExpr::List(mut pair) = binding {
                            let value = pair.pop().expect("let binding {i} must have two elements");
                            let key = pair.pop().expect("let binding {i} must have two elements");

                            if let SExpr::Symbol(name) = key {
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
                Some(SExpr::Symbol(name)) => {
                    let args: Vec<SExpr> = it.collect();

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
