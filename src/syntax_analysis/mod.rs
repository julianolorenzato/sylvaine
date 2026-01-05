mod desugar;
mod lower;
mod macro_expansion;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LispParser;

#[derive(Debug, Clone)]
pub enum SExpr {
    Integer(i32),
    Float(f64),
    Nil,
    Symbol(String),
    List(Vec<SExpr>),
    Prog(Vec<SExpr>),
}

// After, I should implement '(1 2 3) as a syntax sugar to (quote (1 2 3))
pub fn parse(input: String) -> lower::Expr {
    match LispParser::parse(Rule::program, &input) {
        Ok(mut pairs) => {
            let program = pairs.next().expect("Erro ao fazer o parse");

            // source parsing
            let s_expr_ast = build_s_expression_ast(program);
            // desugarizing
            let desugarized_s_expr_ast = desugar::desugar(s_expr_ast);
            // lowering
            let lower_expr_ast = lower::lower(desugarized_s_expr_ast);

            lower_expr_ast
        }

        Err(_err) => panic!("parse error"),
    }
}

fn build_s_expression_ast(pair: Pair<Rule>) -> SExpr {
    match pair.as_rule() {
        Rule::program => {
            let exprs = pair.into_inner().map(build_s_expression_ast).collect();
            SExpr::Prog(exprs)
        }
        Rule::symbol => SExpr::Symbol(pair.as_str().to_string()),
        Rule::nil => SExpr::Nil,
        Rule::integer => SExpr::Integer(pair.as_str().parse().unwrap()),
        Rule::float => SExpr::Float(pair.as_str().parse().unwrap()),
        Rule::list => {
            let list = pair.into_inner().map(build_s_expression_ast).collect();
            SExpr::List(list)
        }
        _ => unreachable!(),
    }
}
