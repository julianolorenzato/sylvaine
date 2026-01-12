use std::{
    collections::HashMap,
    ops::{Bound, Sub},
};

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

// type LambdaDict = HashMap<*const Expr, String>;

// fn collect_lambdas(node: &Expr, dict: &mut LambdaDict) {
//     match node {
//         Expr::Lambda(params, body) => {
//             let func_id = format!("fn_{}", dict.len());

//             dict.insert(node as *const Expr, func_id);

//             collect_lambdas(body, dict);
//         },
//         Expr::Define(_, expr)
//     }
// }

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
        Expr::Define(name, expr) => {
            let global_idx = wasm.sections.globals.len();
            wasm.sections.globals.global(
                GlobalType {
                    val_type: LISP_OBJ_VAL_TYPE,
                    mutable: true,
                    shared: false,
                },
                &ConstExpr::empty(),
            );
            compile(expr, wasm, env);

            wasm.sections.functions.function(MAIN_FUNC_SIG_TYPE_IDX);
            wasm.current_func.instructions().global_set(global_idx);
            wasm.sections.code.function(&wasm.current_func);
        }
        Expr::Float(n) => {
            wasm.float_value(*n);
        }
        Expr::Integer(n) => {
            wasm.integer_value(*n);
        }
        Expr::Symbol(name) => {
            let (var, scope_index) = env.resolve(&name).expect("symbol not found");

            if scope_index != 0 {
                match var {
                    Var::Bound(i) => {
                        env.define_free(name.to_string(), i, scope_index);
                    }
                    Var::Free(_, _) => unreachable!(),
                }
            } else {
                match var {
                    Var::Bound(i) => {}
                    Var::Free(a, b) => {
                        wasm.current_func.instructions().local_get(a);
                    }
                }
            }
        }
        Expr::Lambda(params, body) => {
            let func_idx = wasm.sections.functions.len();

            let ctx: HashMap<String, u32> = HashMap::new();

            wasm.current_func = Function::new([]);

            env.push_scope();

            for (i, param) in params.iter().enumerate() {
                env.define_bound(param.to_string(), i as u32);
            }

            compile(body, wasm, env);

            env.pop_scope();

            wasm.sections.functions.function(LISP_FUNC_SIG_TYPE_IDX);
            wasm.current_func.instruction(&Instruction::End);
            wasm.sections.code.function(&wasm.current_func);

            // after, just left a $Closure object on the stack of the current flow
            let mut ctx_fields = vec![];
            for (name, idx) in ctx {
                ctx_fields.push(FieldType {
                    element_type: StorageType::Val(LISP_OBJ_VAL_TYPE),
                    mutable: false,
                });

                wasm.current_func.instruction(&Instruction::LocalGet(idx));
            }

            wasm.sections.types.ty().struct_(ctx_fields);

            let ctx_type_idx = wasm.sections.types.len() - 1;

            wasm.current_func
                .instructions()
                .struct_new(ctx_type_idx)
                .ref_func(func_idx)
                .struct_new(CLOSURE_TYPE_IDX);
        }
        Expr::Call(expr, args) => {
            wasm.current_func
                .instructions()
                .ref_null(LISP_OBJ_HEAP_TYPE)
                .struct_new(LISP_OBJ_TYPE_IDX);
            compile(expr, wasm, env);

            for arg in args.iter().rev() {
                compile(arg, wasm, env);
            }

            compile(expr, wasm, env);

            wasm.current_func
                .instructions()
                // .local_get(0)
                // .struct_get(LISP_OBJ_TYPE_IDX, 0)
                // .local_get(1)
                // .struct_get(CLOSURE_TYPE_IDX, CLOSURE_FIELD_ENV_TYPE_IDX)
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
        Expr::List(items) => todo!(),
    }
}

struct WasmCode {
    module: Module,
    current_func: Function,
    current_func_n_params: u32,
    sections: WasmCodeSections,
    current_func_idx: u32,
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

// TYPE INDEXES:
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

// Cons cell
const CONS_CELL_TYPE_IDX: u32 = 2;
const CONS_CELL_HEAP_TYPE: HeapType = HeapType::Concrete(CONS_CELL_TYPE_IDX);

const CONS_CELL_FIELD_CAR_IDX: u32 = 0;
const CONS_CELL_FIELD_CDR_IDX: u32 = 1;

// Integer
const INTEGER_TYPE_IDX: u32 = 3;
const INTEGER_HEAP_TYPE: HeapType = HeapType::Concrete(INTEGER_TYPE_IDX);

// Float
const FLOAT_TYPE_IDX: u32 = 4;

// Function types
const LISP_FUNC_SIG_TYPE_IDX: u32 = 5;
const LISP_FUNC_SIG_LOCAL_ENV: u32 = 0;
const LISP_FUNC_SIG_LOCAL_ARGS: u32 = 1;

const INIT_FUNC_SIG_TYPE_IDX: u32 = 6;
const MAIN_FUNC_SIG_TYPE_IDX: u32 = 7;

// Function INDEXES:
const INIT_FUNC_IDX: u32 = 0;
const MAIN_FUNC_IDX: u32 = 1;
const ADD_FUNC_IDX: u32 = 2;

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

        // Defining Lisp Func Signature
        types.ty().func_type(&FuncType::new(
            [LISP_OBJ_VAL_TYPE, LISP_OBJ_VAL_TYPE],
            [LISP_OBJ_VAL_TYPE],
        ));

        // Defining Init Func Signature
        types.ty().func_type(&FuncType::new([], []));

        // Defining Main Func Signature
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
            current_func_idx: 3,
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

    fn open_function(&mut self) {
        self.current_func = Function::new([]);
        // self.current_func_n_params = n_params;
    }

    fn close_function(&mut self, func_idx: u32) {
        // let mut func_params = vec![];
        // for _ in 0..self.current_func_n_params {
        //     func_params.push(ValType::I32);
        // }

        self.sections.functions.function(LISP_FUNC_SIG_TYPE_IDX);
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

    fn generate_unique_label() -> String {
        random_hash(12)
    }
}

#[derive(Clone, Copy)]
enum Var {
    Bound(u32),
    Free(u32, u32),
}

struct Environment {
    scopes: Vec<HashMap<String, Var>>,
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
        }
    }

    fn scope_level(&self) -> u32 {
        (self.scopes.len() - 1) as u32
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scope_level() != 0 {
            self.scopes.pop();
        }
    }

    fn define_bound(&mut self, name: String, index: u32) -> u32 {
        let idx = (self.scopes.len() + 1) as u32;

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, Var::Bound(index));
        } else {
            panic!("define local")
        }

        idx
    }

    fn define_free(&mut self, name: String, index: u32, scope_index: u32) -> u32 {
        let idx = (self.scopes.len() + 1) as u32;

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, Var::Free(index, scope_index));
        } else {
            panic!("define free")
        }

        idx
    }

    fn resolve(&mut self, name: &str) -> Option<(Var, u32)> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if let Some(var) = scope.get(name) {
                return Some((*var, i as u32));
            }
        }
        None
    }
}
