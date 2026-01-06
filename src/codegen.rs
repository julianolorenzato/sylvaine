use std::collections::HashMap;

use crate::syntax_analysis::lower::Expr;
use rand::Rng;
use wasm_encoder::{
    AbstractHeapType, CodeSection, CompositeInnerType, CompositeType, ConstExpr, EntityType,
    FieldType, FuncType, Function, FunctionSection, GlobalSection, GlobalType, HeapType, Ieee64,
    ImportSection, Instruction, Module, NameMap, NameSection, RefType, StorageType, StructType,
    SubType, TableSection, TableType, TypeSection, ValType,
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

pub fn codegen(ast: &Expr) -> Vec<u8> {
    let mut wasm_code = WasmCode::new();
    let mut env = Environment::new();

    compile(ast, &mut wasm_code, &mut env);

    let bin_wasm = wasm_code.finish();

    let wat = wasmprinter::print_bytes(&bin_wasm).unwrap();

    println!("{wat}");

    bin_wasm
}

fn compile(node: &Expr, wasm: &mut WasmCode, env: &mut Environment) {
    match node {
        Expr::Define(name, expr) => match *expr.clone() {
            Expr::Lambda(params, body) => {
                env.push_scope();

                for param in params {
                    env.define_local(param, true);
                }
                compile(&body, wasm, env);

                env.pop_scope();

                env.define_local(name.to_string(), true);
            }
            _ => todo!(),
        },
        Expr::Float(n) => {
            wasm.float_value(*n);
        }
        Expr::Integer(n) => {
            wasm.integer_value(*n);
        }
        Expr::Symbol(name) => {
            let scope_index = env.resolve(&name).expect("symbol not found");

            // Check if its a free variable
            // Free var: (struct.get ...)
            // Bound var: (local.get $name)

            match (&env.current_closure_context, scope_index) {
                // Isn't a free variable
                (None, 0) => {}
                (None, x) => unreachable!(),
                (Some(cc), 0) => {}
                // Its a free variable, need to put inside an environment
                (Some(cc), x) => {
                    // Struct.get
                }
            }
        }
        Expr::Call(name, args) => {
            wasm.current_func.instructions().call_ref(0);
        }
        Expr::Let(bindings, body) => {
            todo!()
        }
        Expr::Prog(expressions) => {
            wasm.open_function(0);

            for expression in expressions {
                compile(expression, wasm, env);
            }

            wasm.close_function();
        }
        Expr::Lambda(params, body) => {
            // let mut func_params = vec![];
            // for param in params {
            //     func_params.push(ValType::I32);
            // }

            // let func_type = FuncType::new(func_params, [ValType::I32]);

            // types.ty().func_type(&func_type);

            // let type_idx = types.len() - 1;

            compile(body, wasm, env);
        }
        Expr::List(items) => todo!(),
    }
}

fn compile_lambda(params: Vec<String>, body: Expr, wasm: &mut WasmCode, env: &mut Environment) {
    env.push_scope();

    for param in params {
        // All params goes to function
        env.define_local(param, true);
    }

    let closure_env_type = ValType::Ref(RefType {
        nullable: false,
        heap_type: HeapType::ANY,
    });

    FuncType::new([closure_env_type], []);

    compile(&body, wasm, env);

    env.pop_scope();
}

// fn compile_expr(
//     ast: &Expr,
//     env: &mut Environment,
//     types: &mut TypeSection,
//     funcs: &mut FunctionSection,
//     code: &mut CodeSection,
//     current_func: Option<&mut Function>,
// ) {
//     match ast {
//         Expr::List(xs) => match xs.as_slice() {
//             // [Expr::Symbol(s), ..] if s == "define" && env.scope_level() != 0 => {
//             //     panic!("'define' is a special expression that extends (mutates) the current scope, therefore it can only be used at the file top level")
//             // }
//             // // define
//             // [Expr::Symbol(s), Expr::Symbol(name), expr]
//             //     if s == "define" && env.scope_level() == 0 =>
//             // {
//             //     env.define_local(name.to_string(), ValType::I32);
//             // }
//             // let bindings
//             [Expr::Symbol(s), Expr::List(bindings)] if s == "let" => {}
//             // lambda creation
//             [Expr::Symbol(s), Expr::List(params), body] if s == "lambda" => {
//                 env.push_scope();

//                 let typed_params = vec![ValType::I32; params.len()];
//                 types
//                     .ty()
//                     .func_type(&FuncType::new(typed_params, [ValType::I32]));

//                 funcs.function(env.lambdas_count);

//                 let mut fun = Function::new([]);
//                 code.function(&fun);

//                 for param in params {
//                     if let Expr::Symbol(s) = param {
//                         env.define_local(s.to_string(), ValType::I32);
//                     } else {
//                         panic!("param should be a symbol")
//                     }
//                 }

//                 compile_expr(body, env, types, funcs, code, Some(&mut fun));

//                 env.pop_scope();
//             }
//             // lambda call
//             [Expr::Symbol(s), args @ ..] => {
//                 if let Some(symbol) = env.resolve(s) {
//                     if let Some(func) = current_func {
//                         func.instruction(Instruction::CallIndirect {
//                             type_index: (),
//                             table_index: (),
//                         })
//                     }
//                 } else {
//                     panic!("identifier {} not found", s);
//                 }
//             }
//             _ => todo!(),
//         },
//         Expr::Integer(n) => {
//             // if
//             // current_fun.instruction(&Instruction::I32Const(*n));
//         }
//         _ => todo!(),
//     }
// }

struct WasmCode {
    module: Module,
    current_func: Function,
    current_func_n_params: u32,
    sections: WasmCodeSections,
}

struct WasmCodeSections {
    types: TypeSection,
    functions: FunctionSection,
    tables: TableSection,
    globals: GlobalSection,
    code: CodeSection,
    names: NameSection,
    imports: ImportSection,
}

const LISP_OBJ_IDX: u32 = 0;
const LISP_OBJ: HeapType = HeapType::Concrete(LISP_OBJ_IDX);

impl WasmCode {
    fn new() -> Self {
        let module = Module::new();
        let mut types = TypeSection::new();
        let mut functions = FunctionSection::new();
        let mut tables = TableSection::new();
        let mut globals = GlobalSection::new();
        let mut code = CodeSection::new();
        let mut names = NameSection::new();
        let mut imports = ImportSection::new();

        names.module("sylvaine_generated");

        // globals.global(
        //     GlobalType {
        //         val_type: ValType::I32,
        //         mutable: false,
        //         shared: false,
        //     },
        //     &ConstExpr::i32_const(34),
        // );

        // Create closure table
        // tables.table(TableType {
        //     element_type: RefType::FUNCREF,
        //     table64: true,
        //     minimum: 1,
        //     maximum: None,
        //     shared: false,
        // });

        // Mathematical functions imports
        for operator in ["+", "-", "*", "/"] {
            imports.import("stdlib", operator, EntityType::Function(0));
        }

        // Naming functions
        let mut function_names = NameMap::new();
        function_names.append(0, "+");
        function_names.append(1, "-");
        function_names.append(2, "*");
        function_names.append(3, "/");
        names.functions(&function_names);

        // Lisp object type
        types.ty().subtype(&SubType {
            is_final: false,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: CompositeInnerType::Struct(StructType {
                    fields: vec![].into(),
                }),
                shared: false,
                descriptor: None,
                describes: None,
            },
        });

        // Closure type
        types.ty().subtype(&SubType {
            is_final: true,
            supertype_idx: Some(LISP_OBJ_IDX),
            composite_type: CompositeType {
                inner: CompositeInnerType::Struct(StructType {
                    fields: vec![
                        FieldType {
                            element_type: StorageType::Val(ValType::Ref(RefType {
                                nullable: false,
                                heap_type: HeapType::Abstract {
                                    shared: false,
                                    ty: AbstractHeapType::Func,
                                },
                            })),
                            mutable: false,
                        },
                        FieldType {
                            element_type: StorageType::Val(ValType::Ref(RefType {
                                nullable: false,
                                heap_type: LISP_OBJ,
                            })),
                            mutable: false,
                        },
                    ]
                    .into(),
                }),
                shared: false,
                descriptor: None,
                describes: None,
            },
        });

        // Mathematical function type
        types
            .ty()
            .func_type(&FuncType::new([ValType::I32, ValType::I32], [ValType::I32]));

        // Naming types
        let mut type_names = NameMap::new();
        type_names.append(0, "lisp_obj");
        type_names.append(1, "closure");
        type_names.append(2, "i32-i32-i32");
        type_names.append(3, "main");
        names.types(&type_names);

        // Define functions
        // functions.function(0);

        // Define functions body
        // let mut main_function = Function::new([]);
        // main_function.instruction(&Instruction::End);

        // code.function(&main_function);

        Self {
            module,
            current_func: Function::new([]),
            current_func_n_params: 0,
            sections: WasmCodeSections {
                types,
                functions,
                tables,
                globals,
                code,
                names,
                imports,
            },
        }
    }

    fn finish(mut self) -> Vec<u8> {
        self.module
            .section(&self.sections.types)
            .section(&self.sections.imports)
            .section(&self.sections.functions)
            .section(&self.sections.tables)
            .section(&self.sections.globals)
            .section(&self.sections.code)
            .section(&self.sections.names);

        self.module.finish()
    }

    fn open_function(&mut self, n_params: u32) {
        self.current_func = Function::new([]);
        self.current_func_n_params = n_params;
    }

    fn close_function(&mut self) {
        let mut func_params = vec![];
        for _ in 0..self.current_func_n_params {
            func_params.push(ValType::I32);
        }

        self.sections
            .types
            .ty()
            .func_type(&FuncType::new(func_params, [ValType::I32]));

        let type_index = self.sections.types.len() - 1;

        self.sections.functions.function(type_index);

        self.current_func.instruction(&Instruction::End);
        self.sections.code.function(&self.current_func);
    }

    fn float_value(&mut self, n: f64) {
        let value = Ieee64::new(n.to_bits());
        let instruction = Instruction::F64Const(value);
        self.current_func.instruction(&instruction);
    }

    fn integer_value(&mut self, n: i32) {
        let instruction = Instruction::I32Const(n);
        self.current_func.instruction(&instruction);
    }
}

struct Environment {
    scopes: Vec<HashMap<String, bool>>,
    lambdas_count: u32,
    current_closure_context: Option<Vec<String>>,
}

#[derive(Clone)]
enum Type {
    Integer(i32),
    Float(f64),
    Function(Vec<Type>, Box<Type>),
    List(Vec<Type>),
}

impl Environment {
    fn new() -> Self {
        Self {
            // intialize with the top level scope
            scopes: vec![HashMap::new()],
            lambdas_count: 0,
            current_closure_context: None,
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

    fn define_local(&mut self, name: String, ty: bool) -> u32 {
        let idx = (self.scopes.len() + 1) as u32;

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        } else {
            panic!("define local")
        }

        idx
    }

    fn resolve(&self, name: &str) -> Option<usize> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if let Some(info) = scope.get(name) {
                return Some(i);
            }
        }
        None
    }
}
