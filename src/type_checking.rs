use std::collections::HashMap;

use crate::lowering::Expr;

#[derive(Clone)]
enum Type {
    Integer(i32),
    Float(f64),
    Bool(bool),
    Lambda(Vec<Type>, Box<Type>),
    Void,
    Unknown,
}

struct TypedExpr {
    expr: Expr,
    ty: Type,
}

// enum TypedExpr {
//     Atom(Type),
//     Call(String, Vec<TypedExpr>, Type),
//     Lambda(Vec<String>, Box<TypedExpr>, Type),
//     Define(String, Box<TypedExpr>, Type),
//     Let(Vec<(String, TypedExpr)>, Type),
//     // List(Vec<TypedExpr>, Type),
// }

#[derive(Clone)]
struct NameInfo {
    ty: Type,
    // scope: u32,
}

struct Environment {
    scopes: Vec<HashMap<String, NameInfo>>,
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
                scope.insert(name.to_string(), NameInfo { ty });
            }
        } else {
            unreachable!("there is no scope level")
        }
    }

    fn resolve(&self, name: &str) -> Option<NameInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info.clone());
            }
        }

        None
    }
}

pub fn check(ast: Expr) -> TypedExpr {
    let mut env = Environment::new();

    analyze(ast, &mut env)
}

fn analyze(node: Expr, env: &mut Environment) -> TypedExpr {
    match node {
        Expr::Symbol(name) => {
            if let Some(name_info) = env.resolve(&name) {
                TypedExpr { expr: Expr::Symbol(name.clone()), ty: name_info.ty }
            } else {
                panic!("identifier {} not found", name);
            }
        }
        Expr::Integer(n) => TypedExpr {
            expr: node.clone(),
            ty: Type::Integer(n),
        },
        Expr::Float(n) => TypedExpr {
            expr: node.clone(),
            ty: Type::Float(n),
        },
        Expr::Define(ref name, ref expr) => {
            let typed_expr = analyze(*expr.clone(), env);

            env.define_local(name, typed_expr.ty);

            TypedExpr {
                expr: node,
                ty: Type::Void,
            }
        }
        Expr::Lambda(params, body) => {
            todo!()
        }
        Expr::Let(ref bindings, ref body) => {
            env.push_scope();

            // Sequential Let, bindings can use previous binding in their expressions
            for (name, ref expr) in bindings {
                let typed_expr = analyze(expr.clone(), env);

                env.define_local(&name, typed_expr.ty);
            }

            let typed_body = analyze(*body.clone(), env);

            env.pop_scope();

            TypedExpr {
                expr: node,
                ty: typed_body.ty,
            }
        }
        Expr::Call(name, args) => todo!(),
    }
}

// fn analyze(node: &Expr, env: &mut Environment) -> TypedExpr {
//     match node {
//         Expr::Symbol(name) => {
//             if let Some(name_info) = env.resolve(&name) {
//                 TypedExpr { expr: Expr::Symbol(name.clone()), ty: name_info.ty }
//             } else {
//                 panic!("identifier {} not found", name);
//             }
//         }
//         Expr::Integer(n) => TypedExpr {
//             expr: node.clone(),
//             ty: Type::Integer(*n),
//         },
//         Expr::Float(n) => TypedExpr {
//             expr: node.clone(),
//             ty: Type::Float(*n),
//         },
//         Expr::Define(name, expr) => {
//             let typed_expr = analyze(expr, env);

//             env.define_local(&name, typed_expr.ty);

//             TypedExpr {
//                 expr: node.clone(),
//                 ty: Type::Void,
//             }
//         }
//         Expr::Lambda(params, body) => {
//             todo!()
//         }
//         Expr::Let(bindings, body) => {
//             env.push_scope();

//             // Sequential Let, bindings can use previous binding in their expressions
//             for (name, expr) in bindings {
//                 let typed_expr = analyze(expr, env);

//                 env.define_local(&name, typed_expr.ty);
//             }

//             let typed_body = analyze(body, env);

//             env.pop_scope();

//             TypedExpr {
//                 expr: node.clone(),
//                 ty: typed_body.ty,
//             }
//         }
//         Expr::Call(name, args) => todo!(),
//     }
// }

// fn analyze(ast: &Expr, env: &mut Environment) -> TypedExpr {
//     match Expr {
//         Expr::Integer(n) => TypedExpr {
//             expr: Expr::Integer(n),
//             ty: Type::Int(n),
//         },
//         Expr::List(items) => match items.as_slice() {
//             [Expr::Symbol(s), Expr::Symbol(name), expr] if s == "define" => {
//                 let typed_expr = analyze(expr, env);

//                 env.define_local(name, typed_expr.ty);

//                 // TypedExpr::Define(name.to_string(), Box::new(typed_expr), Type::Void)

//                 TypedExpr {
//                     expr: Expr::List(items),
//                     ty: Type::Void,
//                 }
//             }
//             [Expr::Symbol(s), Expr::List(params), body] if s == "lambda" => {
//                 // let lambda_params = vec![];
//                 // for param in params {
//                 //     if let Expr::Symbol(param_name) = param {
//                 //         lambda_params.push((param_name.to_string(), Type::Unknown));
//                 //     } else {
//                 //         panic!("lambda param must be an atom")
//                 //     }
//                 // }

//                 // TypedExpr::Lambda(lambda_params, Box::new(analyze(body, env)))

//                 TypedExpr { expr: (), ty: () }
//             }
//             [Expr::Symbol(s), Expr::List(bindings)] if s == "let" => {
//                 let let_bindings = vec![];
//                 for binding in bindings {
//                     if let Expr::List(elements) = binding {
//                         match elements.as_slice() {
//                             [Expr::Symbol(name), expr] => {
//                                 let_bindings.push((name.to_string(), analyze(expr, env)));
//                             }
//                             _ => panic!("each let binding must be a pair of name/expression"),
//                         }
//                     } else {
//                         panic!("the let argument must be a list")
//                     }
//                 }

//                 TypedExpr::Let(let_bindings)
//             }
//         },
//         Expr::Symbol(name) => {
//             if let Some(name_info) = env.resolve(name.as_str()) {
//                 match name_info.ty {
//                     Type::Lambda(params_types, return_type) => {
//                         TypedExpr::Lambda(params_types, return_type)
//                     }
//                     t => TypedExpr::Atom(t),
//                 }
//             } else {
//                 panic!("identifier {} not found", name)
//             }
//         }
//         _ => todo!(),
//     }
// }
