use std::{collections::HashMap, ops::Sub};

use crate::syntax_analysis::lower::Expr;
use rand::Rng;
use wasm_encoder::{
    AbstractHeapType, CodeSection, CompositeInnerType, CompositeType, ConstExpr, EntityType,
    ExportKind, ExportSection, FieldType, FuncType, Function, FunctionSection, GlobalSection,
    GlobalType, HeapType, Ieee64, ImportSection, IndirectNameMap, Instruction, Module, NameMap,
    NameSection, RefType, StartSection, StorageType, StructType, SubType, TableSection, TableType,
    TypeSection, ValType,
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
            wasm.current_func
                .instructions()
                .local_get(0)
                .struct_get(LISP_OBJ_TYPE_IDX, 0)
                .local_get(1)
                .struct_get(CLOSURE_TYPE_IDX, CLOSURE_FIELD_ENV_TYPE_IDX)
                .call_ref(LISP_FUNC_SIG_TYPE_IDX);
        }
        Expr::Let(bindings, body) => {
            todo!()
        }
        Expr::Prog(expressions) => {
            // wasm.open_function(0);

            for expression in expressions {
                compile(expression, wasm, env);
            }

            // wasm.close_function();
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
    start: StartSection,
    exports: ExportSection,
}

// Lisp obj
const LISP_OBJ_TYPE_IDX: u32 = 0;
const LISP_OBJ_HEAP_TYPE: HeapType = HeapType::Concrete(LISP_OBJ_TYPE_IDX);
const LISP_OBJ_REF_TYPE: RefType = RefType {
    nullable: true,
    heap_type: LISP_OBJ_HEAP_TYPE,
};
const LISP_OBJ_VAL_TYPE: ValType = ValType::Ref(LISP_OBJ_REF_TYPE);

// Closure
const CLOSURE_TYPE_IDX: u32 = 1;
const CLOSURE_HEAP_TYPE: HeapType = HeapType::Concrete(CLOSURE_TYPE_IDX);

const CLOSURE_FIELD_FUNC_TYPE_IDX: u32 = 0;
const CLOSURE_FIELD_FUNC_HEAP_TYPE: HeapType = HeapType::Abstract {
    shared: false,
    ty: AbstractHeapType::Func,
};
const CLOSURE_FIELD_FUNC_REF_TYPE: RefType = RefType {
    nullable: false,
    heap_type: CLOSURE_FIELD_FUNC_HEAP_TYPE,
};

const CLOSURE_FIELD_ENV_TYPE_IDX: u32 = 1;

const CONS_CELL_TYPE_IDX: u32 = 2;
const CONS_CELL_HEAP_TYPE: HeapType = HeapType::Concrete(CONS_CELL_TYPE_IDX);

const CONS_CELL_FIELD_CAR_IDX: u32 = 0;
const CONS_CELL_FIELD_CDR_IDX: u32 = 1;

const INTEGER_TYPE_IDX: u32 = 3;
const INTEGER_HEAP_TYPE: HeapType = HeapType::Concrete(INTEGER_TYPE_IDX);

const FLOAT_TYPE_IDX: u32 = 4;

const LISP_FUNC_SIG_TYPE_IDX: u32 = 5;
const LISP_FUNC_SIG_LOCAL_ENV: u32 = 0;
const LISP_FUNC_SIG_LOCAL_ARGS: u32 = 1;

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
        // for operator in ["+", "-", "*", "/"] {
        //     imports.import("stdlib", operator, EntityType::Function(0));
        // }

        // Naming functions
        // let mut function_names = NameMap::new();
        // function_names.append(0, "+");
        // function_names.append(1, "-");
        // function_names.append(2, "*");
        // function_names.append(3, "/");
        // names.functions(&function_names);

        // Defining Lisp Obj type
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

        // Defining Closure type
        types.ty().subtype(&SubType {
            is_final: true,
            supertype_idx: Some(LISP_OBJ_TYPE_IDX),
            composite_type: CompositeType {
                inner: CompositeInnerType::Struct(StructType {
                    fields: vec![
                        FieldType {
                            element_type: StorageType::Val(ValType::Ref(
                                CLOSURE_FIELD_FUNC_REF_TYPE,
                            )),
                            mutable: false,
                        },
                        FieldType {
                            element_type: StorageType::Val(LISP_OBJ_VAL_TYPE),
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

        // Naming Closure fields
        let mut field_names = IndirectNameMap::new();
        let mut closure_field_names = NameMap::new();
        closure_field_names.append(0, "func");
        closure_field_names.append(1, "env");

        field_names.append(CLOSURE_TYPE_IDX, &closure_field_names);
        names.fields(&field_names);

        // Defining Cons Cell
        types.ty().subtype(&SubType {
            is_final: true,
            supertype_idx: Some(LISP_OBJ_TYPE_IDX),
            composite_type: CompositeType {
                inner: CompositeInnerType::Struct(StructType {
                    fields: vec![
                        FieldType {
                            element_type: StorageType::Val(LISP_OBJ_VAL_TYPE),
                            mutable: false,
                        },
                        FieldType {
                            element_type: StorageType::Val(LISP_OBJ_VAL_TYPE),
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

        // Naming Cons Cell fields
        let mut field_names = IndirectNameMap::new();
        let mut cons_cell_field_names = NameMap::new();
        cons_cell_field_names.append(CONS_CELL_FIELD_CAR_IDX, "car");
        cons_cell_field_names.append(CONS_CELL_FIELD_CDR_IDX, "cdr");
        field_names.append(CONS_CELL_TYPE_IDX, &cons_cell_field_names);
        names.fields(&field_names);

        // Defining Integer
        types.ty().subtype(&SubType {
            is_final: true,
            supertype_idx: Some(LISP_OBJ_TYPE_IDX),
            composite_type: CompositeType {
                inner: CompositeInnerType::Struct(StructType {
                    fields: vec![FieldType {
                        element_type: StorageType::Val(ValType::I32),
                        mutable: false,
                    }]
                    .into(),
                }),
                shared: false,
                descriptor: None,
                describes: None,
            },
        });

        // Defining Float
        types.ty().subtype(&SubType {
            is_final: true,
            supertype_idx: Some(LISP_OBJ_TYPE_IDX),
            composite_type: CompositeType {
                inner: CompositeInnerType::Struct(StructType {
                    fields: vec![FieldType {
                        element_type: StorageType::Val(ValType::F64),
                        mutable: false,
                    }]
                    .into(),
                }),
                shared: false,
                descriptor: None,
                describes: None,
            },
        });

        // Defining Lisp Func Sig
        types.ty().func_type(&FuncType::new(
            [LISP_OBJ_VAL_TYPE, LISP_OBJ_VAL_TYPE],
            [LISP_OBJ_VAL_TYPE],
        ));

        // Defining Main Sig
        types.ty().func_type(&FuncType::new([], [ValType::I32]));

        // Naming types
        let mut type_names = NameMap::new();
        type_names.append(LISP_OBJ_TYPE_IDX, "lisp_obj");
        type_names.append(CLOSURE_TYPE_IDX, "closure");
        type_names.append(CONS_CELL_TYPE_IDX, "cons_cell");
        type_names.append(INTEGER_TYPE_IDX, "integer");
        type_names.append(FLOAT_TYPE_IDX, "float");
        type_names.append(LISP_FUNC_SIG_TYPE_IDX, "lisp_func_sig");
        type_names.append(6, "main");
        names.types(&type_names);

        // Define functions
        functions.function(LISP_FUNC_SIG_TYPE_IDX);
        let mut builtin_function_plus = Function::new([]);
        builtin_function_plus
            .instructions()

            // Pega o car
            .local_get(LISP_FUNC_SIG_LOCAL_ARGS)
            .ref_cast_non_null(CONS_CELL_HEAP_TYPE)
            .struct_get(CONS_CELL_TYPE_IDX, CONS_CELL_FIELD_CAR_IDX)

            // pega o inteiro dentro do car
            .ref_cast_non_null(INTEGER_HEAP_TYPE)
            .struct_get(INTEGER_TYPE_IDX, 0)

            // pega o cdr
            .local_get(LISP_FUNC_SIG_LOCAL_ARGS)
            .ref_cast_non_null(CONS_CELL_HEAP_TYPE)
            .struct_get(CONS_CELL_TYPE_IDX, CONS_CELL_FIELD_CDR_IDX)

            // pega o car do cdr
            .ref_cast_non_null(CONS_CELL_HEAP_TYPE)
            .struct_get(CONS_CELL_TYPE_IDX, CONS_CELL_FIELD_CAR_IDX)

            // o outro inteiro do outro car
            .ref_cast_non_null(INTEGER_HEAP_TYPE)
            .struct_get(INTEGER_TYPE_IDX, 0)

            // Soma os dois inteiros da pilha
            .i32_add()

            // Empacota o inteiro resultante da operação em uma nova box
            .struct_new(INTEGER_TYPE_IDX)

            // Finaliza o código da função
            .end();
        code.function(&builtin_function_plus);

        functions.function(6);
        let mut main_function = Function::new([]);
        main_function
            .instructions()
            
            .ref_null(LISP_OBJ_HEAP_TYPE)

            .i32_const(30)
            .struct_new(INTEGER_TYPE_IDX)

            .i32_const(24)
            .struct_new(INTEGER_TYPE_IDX)
            .ref_null(LISP_OBJ_HEAP_TYPE)
            .struct_new(CONS_CELL_TYPE_IDX)

            .struct_new(CONS_CELL_TYPE_IDX)
            
            .call(0)

            .ref_cast_non_null(INTEGER_HEAP_TYPE)
            .struct_get(INTEGER_TYPE_IDX, 0)
            .end();
        code.function(&main_function);

        let start = StartSection { function_index: 1 };

        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 1);

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
                start,
                exports,
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
            .section(&self.sections.exports)
            // .section(&self.sections.start)
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
