use crate::lowering::Expr;
use rand::Rng;
use std::{
    collections::HashMap,
    fs,
};
use wasm_encoder::{
    CodeSection, ConstExpr, FuncType, Function, FunctionSection, GlobalSection, GlobalType,
    Instruction, Module, NameMap, NameSection, RefType, TableSection, TableType, TypeSection,
    ValType,
};

#[derive(Debug, Clone)]
enum Kind {
    Lambda,
    Integer,
}

fn random_hash(len: usize) -> String {
    let mut rng = rand::rng();
    (0..len)
        .map(|_| format!("{:02x}", rng.random::<u8>()))
        .collect()
}

#[derive(Debug, Clone)]
struct Symbol {
    // level: u32,
    kind: Kind,
    // vt: ValType,
    // closure: bool,
}

struct Environment {
    scopes: Vec<HashMap<String, Symbol>>,
    lambdas_count: u32,
}

impl Environment {
    fn new() -> Self {
        Self {
            // intialize with the top level scope
            scopes: vec![HashMap::new()],
            lambdas_count: 0,
        }
    }

    fn scope_level(&self) -> u32 {
        (self.scopes.len() - 1) as u32
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.lambdas_count += 1;
    }

    fn pop_scope(&mut self) {
        if self.scope_level() != 0 {
            self.scopes.pop();
        }
    }

    fn define_local(&mut self, name: String, kind: Kind) -> u32 {
        let idx = (self.scopes.len() + 1) as u32;

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name,
                Symbol {
                    // level: idx,
                    kind,
                },
            );
        } else {
            panic!("define local")
        }

        idx
    }

    fn resolve(&self, name: &str) -> Option<Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info.clone());
            }
        }
        None
    }
}

pub fn codegen(ast: &Expr) {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    let mut functions = FunctionSection::new();
    let mut tables = TableSection::new();
    let mut globals = GlobalSection::new();
    let mut code = CodeSection::new();
    let mut names = NameSection::new();

    names.module("sylvaine_generated");

    globals.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: false,
            shared: false,
        },
        &ConstExpr::i32_const(34),
    );

    // Naming functions
    let mut function_names = NameMap::new();
    function_names.append(0, "main");
    names.functions(&function_names);

    // Create closure table
    tables.table(TableType {
        element_type: RefType::FUNCREF,
        table64: true,
        minimum: 1,
        maximum: None,
        shared: false,
    });

    // Define function types
    types.ty().func_type(&FuncType::new([], [ValType::I32]));
    types
        .ty()
        .func_type(&FuncType::new([ValType::I32, ValType::I32], [ValType::I32]));

    // Define functions
    functions.function(99).function(0);

    // Define functions body
    let mut main_function = Function::new([(2, ValType::F64)]);
    main_function.instruction(&Instruction::End);

    let mut add_function = Function::new([]);
    add_function
        .instruction(&Instruction::LocalGet(0))
        .instruction(&Instruction::LocalGet(1))
        .instruction(&Instruction::I32Add)
        .instruction(&Instruction::End);

    code.function(&main_function).function(&add_function);

    // Generate WASM
    let mut env = Environment::new();
    // compile_top_level(ast, &mut env, &mut types, &mut functions, &mut code);
    // compile_expr(ast, &mut module, &mut env, &mut main_function);

    module
        .section(&types)
        .section(&functions)
        .section(&tables)
        .section(&globals)
        .section(&code)
        .section(&names);

    let bin_wasm = module.finish();

    let wat = wasmprinter::print_bytes(bin_wasm).unwrap();

    println!("{wat}");
}

fn compile_top_level(
    ast: &Expr,
    env: &mut Environment,
    types: &mut TypeSection,
    funcs: &mut FunctionSection,
    code: &mut CodeSection,
) {
    match ast {
        Expr::List(xs) => match xs.as_slice() {
            [Expr::Symbol(s), ..] if s == "define" && env.scope_level() != 0 => {
                panic!("'define' is a special expression that extends (mutates) the current scope, therefore it can only be used at the file top level")
            }
            [Expr::Symbol(s), Expr::Symbol(name), expr]
                if s == "define" && env.scope_level() == 0 =>
            {
                match expr {
                    Expr::List(items) => match items.as_slice() {
                        [Expr::Symbol(s), Expr::List(params), body] if s == "lambda" => {
                            env.define_local(name.to_string(), Kind::Lambda);

                            let typed_params = vec![ValType::I32; params.len()];
                            types
                                .ty()
                                .func_type(&FuncType::new(typed_params, [ValType::I32]));

                            funcs.function(env.lambdas_count);

                            let mut fun = Function::new([]);
                            code.function(&fun);

                            env.push_scope();
                            for param in params {
                                if let Expr::Symbol(s) = param {
                                    env.define_local(s.to_string(), ValType::I32);
                                } else {
                                    panic!("param should be a symbol")
                                }
                            }
                            compile_expr(body, env, types, funcs, code, Some(&mut fun));

                            env.pop_scope();
                        }
                        [Expr::Symbol(s), args @ ..] => {}
                        _ => panic!("cannot"),
                    },
                    Expr::Integer(n) => {}
                    _ => {}
                }

                compile_expr(expr, env, types, funcs, code, None);
            }
            _ => (),
        },
        _ => (),
    }
}

fn compile_expr(
    ast: &Expr,
    env: &mut Environment,
    types: &mut TypeSection,
    funcs: &mut FunctionSection,
    code: &mut CodeSection,
    current_func: Option<&mut Function>,
) {
    match ast {
        Expr::List(xs) => match xs.as_slice() {
            // [Expr::Symbol(s), ..] if s == "define" && env.scope_level() != 0 => {
            //     panic!("'define' is a special expression that extends (mutates) the current scope, therefore it can only be used at the file top level")
            // }
            // // define
            // [Expr::Symbol(s), Expr::Symbol(name), expr]
            //     if s == "define" && env.scope_level() == 0 =>
            // {
            //     env.define_local(name.to_string(), ValType::I32);
            // }
            // let bindings
            [Expr::Symbol(s), Expr::List(bindings)] if s == "let" => {}
            // lambda creation
            [Expr::Symbol(s), Expr::List(params), body] if s == "lambda" => {
                env.push_scope();

                let typed_params = vec![ValType::I32; params.len()];
                types
                    .ty()
                    .func_type(&FuncType::new(typed_params, [ValType::I32]));

                funcs.function(env.lambdas_count);

                let mut fun = Function::new([]);
                code.function(&fun);

                for param in params {
                    if let Expr::Symbol(s) = param {
                        env.define_local(s.to_string(), ValType::I32);
                    } else {
                        panic!("param should be a symbol")
                    }
                }

                compile_expr(body, env, types, funcs, code, Some(&mut fun));

                env.pop_scope();
            }
            // lambda call
            [Expr::Symbol(s), args @ ..] => {
                if let Some(symbol) = env.resolve(s) {
                    if let Some(func) = current_func {
                        func.instruction(Instruction::CallIndirect {
                            type_index: (),
                            table_index: (),
                        })
                    }
                } else {
                    panic!("identifier {} not found", s);
                }
            }
            _ => todo!(),
        },
        Expr::Integer(n) => {
            // if
            // current_fun.instruction(&Instruction::I32Const(*n));
        }
        _ => todo!(),
    }
}
// fn gen_webassembly_code(ast: Expr) -> Vec<u8> {
//     let mut code: Vec<u8> = vec![];

//     let magic: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6D];
//     let version: Vec<u8> = vec![0x01, 0x00, 0x00, 0x00];

//     code.extend(magic);
//     code.extend(version);

//     match ast {
//         Expr::Nil => {
//             code.append(&mut vec![2]);
//         }
//         Expr::Integer(n) => {
//             let bytes: [u8; 4] = n.to_le_bytes();
//             code.push(0x222);
//         }
//         Expr::Float(n) => {
//             todo!();
//         }
//         Expr::Symbol(s) => {
//             todo!();
//         }
//         Expr::List(l) => {
//             todo!();
//         }
//     }
//     vec![2, 3]
// }

pub fn gen_webassembly_code(ast: Expr) -> String {
    // let mut code: Vec<u8> = vec![];

    let mut code: String = String::new();

    code.push_str("(module\n");
    code.push_str("\t(fun $main (result i32)\n");

    traverse_gen(&ast, &mut code);

    code.push_str("\t)\n");
    code.push_str("\t(export \"main\" (func $main))\n");
    code.push_str(")\n");

    fs::write("output.wat", &code).unwrap();

    code
}

fn insert_headers(code: &mut String) {
    // let magic: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6D];
    // let version: Vec<u8> = vec![0x01, 0x00, 0x00, 0x00];

    // code.extend(magic);
    // code.extend(version);
}

fn traverse_gen(ast: &Expr, code: &mut String) {
    if let Expr::List(funcs) = ast {
        for ast in funcs {
            match ast {
                Expr::Nil => (),
                Expr::Float(n) => code.push_str(n.to_string().as_str()),
                Expr::Integer(n) => code.push_str(n.to_string().as_str()),
                Expr::List(xs) => {
                    println!("{:?}", xs);
                    // if xs.len() > 0 {
                    match &xs[0] {
                        Expr::Symbol(s) if s == "define" => {
                            code.push_str(format!("\t(func {} \n", xs[1]).as_str());

                            code.push_str("\t)\n");
                        }
                        Expr::Symbol(s) if s == "quote" => {}
                        a => unreachable!("Tratar este erro de uma melhor forma {a}",),
                    }
                }
                Expr::Symbol(s) => code.push_str(s),
            }
        }
    } else {
        unreachable!()
    }
}
