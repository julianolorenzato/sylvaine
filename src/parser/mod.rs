use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LispParser;

#[derive(Debug)]
pub enum SExpr {
    Integer(i32),
    Float(f64),
    Nil,
    Symbol(String),
    List(Vec<SExpr>),
    Prog(Vec<SExpr>)
}

// After, I should implement '(1 2 3) as a syntax sugar to (quote (1 2 3))
// And (define sum (a b) (+ a b)) as a syntax sugar to (define sum (lambda (a b) (+ a b)))
pub fn parse(input: String) -> Result<SExpr, String> {
    match LispParser::parse(Rule::program, &input) {
        Ok(mut pairs) => match pairs.next() {
            Some(program) => Ok(build_sexpr_ast(program)),
            None => Ok(SExpr::Nil),
        },

        Err(err) => Err("Erro ao fazer o parse".to_string()),
    }
}

fn build_sexpr_ast(pair: Pair<Rule>) -> SExpr {
    match pair.as_rule() {
        Rule::program => {
            let exprs = pair.into_inner().map(build_sexpr_ast).collect();
            SExpr::Prog(exprs)
        }
        Rule::symbol => SExpr::Symbol(pair.as_str().to_string()),
        Rule::nil => SExpr::Nil,
        Rule::integer => SExpr::Integer(pair.as_str().parse().unwrap()),
        Rule::float => SExpr::Float(pair.as_str().parse().unwrap()),
        Rule::list => {
            let list = pair.into_inner().map(build_sexpr_ast).collect();
            SExpr::List(list)
        }
        _ => unreachable!(),
    }
}