use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LispParser;

#[derive(Debug)]
pub enum Expr {
    Integer(i32),
    Float(f64),
    Nil,
    Symbol(String),
    List(Vec<Expr>),
}

use std::fmt;

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Integer(i) => write!(f, "{}", i),
            Expr::Float(x) => write!(f, "{}", x),
            Expr::Nil => write!(f, "nil"),
            Expr::Symbol(s) => write!(f, "{}", s),
            Expr::List(xs) => {
                write!(f, "(")?;
                for (i, expr) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, ")")
            }
        }
    }
}

// After, I should implement '(1 2 3) as a syntax sugar to (quote (1 2 3))
// And (define sum (a b) (+ a b)) as a syntax sugar to (define sum (lambda (a b) (+ a b)))
pub fn parse(input: String) -> Result<Expr, String> {
    match LispParser::parse(Rule::program, &input) {
        Ok(mut pairs) => match pairs.next() {
            Some(program) => Ok(build_ast(program)),
            None => Ok(Expr::Nil),
        },

        Err(err) => Err("Erro ao fazer o parse".to_string()),
    }
}

fn build_ast(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::program => {
            let exprs = pair.into_inner().map(build_ast).collect();
            Expr::List(exprs)
        }
        Rule::symbol => Expr::Symbol(pair.as_str().to_string()),
        Rule::nil => Expr::Nil,
        Rule::integer => Expr::Integer(pair.as_str().parse().unwrap()),
        Rule::float => Expr::Float(pair.as_str().parse().unwrap()),
        Rule::list => {
            let list = pair.into_inner().map(build_ast).collect();
            Expr::List(list)
        }
        _ => unreachable!(),
    }
}
