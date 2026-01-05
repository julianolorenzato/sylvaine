use std::collections::HashMap;

use crate::lowering::Expr;

// Handle error for scope_level != 0 and Define being used

#[derive(Debug, Clone)]
enum Type {
    Integer(i32),
    Float(f64),
    Bool(bool),
    Lambda(Vec<Type>, Box<Type>),
    Void,
    Unknown,
}

#[derive(Debug)]
pub struct TypedExpr {
    expr: Expr,
    return_type: Type,
}

#[derive(Debug)]
pub struct Environment {
    scopes: Vec<HashMap<String, Type>>,
    // symbol_table: HashMap<String, NameInfo>,
    // current_scope: u32,
}

impl Environment {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            // symbol_table: HashMap::new(),
            // current_scope: 0,
        }
    }

    fn scope_level(&self) -> u32 {
        (self.scopes.len() - 1) as u32
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        // self.lambdas_count += 1;
    }

    fn pop_scope(&mut self) {
        if self.scope_level() != 0 {
            self.scopes.pop();
        }
    }

    fn define_local(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(_) = scope.get(name) {
                panic!("this identifier already exists in this scope, shadowing in the same scope is not allowed")
            } else {
                scope.insert(name.to_string(), ty);
            }
        } else {
            unreachable!("there is no scope level")
        }
    }

    fn resolve(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }

        None
    }
}

pub fn check(ast: Expr) -> (TypedExpr, Environment) {
    let mut env = Environment::new();

    (analyze(&ast, &mut env), env)
}

fn analyze(node: &Expr, env: &mut Environment) -> TypedExpr {
    match node {
        Expr::Prog(exprs) => {
            let mut return_type: Type = Type::Void;
            for expr in exprs {
                return_type = analyze(expr, env).return_type;
            }

            // Prog's return type is the return type of last expression or void if it's empty
            TypedExpr {
                expr: node.clone(),
                return_type: return_type,
            }
        }
        Expr::Symbol(name) => {
            if let Some(ty) = env.resolve(&name) {
                TypedExpr {
                    expr: Expr::Symbol(name.clone()),
                    return_type: ty,
                }
            } else {
                panic!("identifier {} not found", name);
            }
        }
        Expr::Integer(n) => TypedExpr {
            expr: node.clone(),
            return_type: Type::Integer(*n),
        },
        Expr::Float(n) => TypedExpr {
            expr: node.clone(),
            return_type: Type::Float(*n),
        },
        Expr::Define(name, expr) => {
            let typed_expr = analyze(expr, env);

            env.define_local(name, typed_expr.return_type);

            TypedExpr {
                expr: node.clone(),
                return_type: Type::Void,
            }
        }
        Expr::Lambda(params, body) => {
            env.push_scope();

            for param in params {
                env.define_local(&param, Type::Unknown);
            }

            let typed_body = analyze(body, env);

            env.pop_scope();

            TypedExpr {
                expr: node.clone(),
                return_type: typed_body.return_type,
            }
        }
        Expr::Let(bindings, body) => {
            env.push_scope();

            // Sequential Let, bindings can use previous binding in their expressions
            for (name, expr) in bindings {
                let typed_expr = analyze(expr, env);

                env.define_local(&name, typed_expr.return_type);
            }

            let typed_body = analyze(body, env);

            env.pop_scope();

            TypedExpr {
                expr: node.clone(),
                return_type: typed_body.return_type,
            }
        }
        Expr::Call(name, _) => {
            if let Some(ty) = env.resolve(name) {
                TypedExpr {
                    expr: node.clone(),
                    return_type: ty,
                }
            } else {
                panic!("identifier '{}' not found", name)
            }
        }
    }
}
