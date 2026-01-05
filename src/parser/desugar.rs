
fn desugar(expr: SExpr) -> SExpr {
    match expr {
        SExpr::List(list) => desugar_list(list),
        SExpr::Prog(exprs) => {
            SExpr::Prog(exprs.into_iter().map(desugar).collect())
        }
        other => other,
    }
}

fn desugar_list(list: Vec<SExpr>) -> SExpr {
    match list.as_slice() {
        // (define f (a b) body)
        [
            SExpr::Symbol(def),
            SExpr::Symbol(name),
            SExpr::List(params),
            body,
        ] if def == "define" => {
            SExpr::List(vec![
                SExpr::Symbol("define".into()),
                SExpr::Symbol(name.clone()),
                SExpr::List(vec![
                    SExpr::Symbol("lambda".into()),
                    SExpr::List(params.clone()),
                    desugar(body.clone()),
                ]),
            ])
        }

        // fallback: desugar recursivamente
        _ => SExpr::List(list.into_iter().map(desugar).collect()),
    }
}
