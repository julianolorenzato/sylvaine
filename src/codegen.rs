use std::{
    collections::{HashMap, HashSet},
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
            // Get the index for a new global
            let global_idx = wasm.sections.globals.len();

            // Create a new null global
            wasm.sections.globals.global(
                GlobalType {
                    val_type: LISP_OBJ_VAL_TYPE,
                    mutable: true,
                    shared: false,
                },
                &ConstExpr::ref_null(LISP_OBJ_HEAP_TYPE),
            );

            // Update the current scope
            env.scopes
                .last_mut()
                .expect("global scope not found")
                .insert(name.to_string(), global_idx);

            // Emit instructions for inner expression
            compile(expr, wasm, env);

            // Emit instructions to set the previous created null global
            wasm.current_func
                .instructions()
                .global_set(global_idx)
                .global_get(global_idx);
        }
        Expr::Float(n) => {
            wasm.current_func
                .instructions()
                .f64_const(Ieee64::new(n.to_bits()))
                .struct_new(FLOAT_TYPE_IDX);
        }
        Expr::Integer(n) => {
            wasm.current_func
                .instructions()
                .i32_const(*n)
                .struct_new(INTEGER_TYPE_IDX);
        }
        Expr::Symbol(name) => {
            match env.resolve(&name) {
                Some(Binding::Argument(idx)) => {
                    wasm.current_func
                        .instructions()
                        .local_get(1) // O Array de Args
                        .i32_const(idx as i32)
                        .array_get(LISP_ARRAY_TYPE_IDX);
                }
                Some(Binding::Captured(idx)) => {
                    wasm.current_func
                        .instructions()
                        .local_get(0) // O Env (Struct)
                        .ref_cast_non_null(HeapType::Concrete(env.current_struct_idx()))
                        .struct_get(env.current_struct_idx(), idx);
                }
                Some(Binding::Global(idx)) => {
                    wasm.current_func.instructions().global_get(idx);
                }
                None => match name.as_str() {
                    "+" => {
                        wasm.current_func.instructions().call(ADD_FUNC_IDX);
                    }
                    "-" => {
                        todo!()
                    }
                    _ => {
                        panic!("symbol {} not found", name);
                    }
                },
            }
        }
        Expr::Lambda(params, body) => {
            let func_idx = wasm.sections.functions.len();
            let env_idx = wasm.sections.types.len();

            let parent_func = std::mem::replace(&mut wasm.current_func, Function::new([]));

            env.push_scope();

            for (i, param) in params.iter().enumerate() {
                env.define(param.to_string(), i as u32);
            }

            compile(body, wasm, env);

            env.pop_scope();

            // Finish lambda code with an End
            wasm.current_func.instruction(&Instruction::End);

            // Return to parent function
            let lambda_func = std::mem::replace(&mut wasm.current_func, parent_func);

            wasm.sections.functions.function(LISP_FUNC_SIG_TYPE_IDX);
            wasm.sections.code.function(&lambda_func);

            // after, just left a $Closure object on the stack of the current flow
            let free_vars = find_free_vars(body, &params);
            let mut captured_values_indices = vec![];
            let mut ctx_fields = vec![];

            for name in &free_vars {
                if let Some(var) = env.resolve(name) {
                    // Se depth > 0, é algo que precisamos capturar do escopo pai
                    captured_values_indices.push((idx, depth, name));

                    ctx_fields.push(FieldType {
                        element_type: StorageType::Val(LISP_OBJ_VAL_TYPE),
                        mutable: false,
                    });
                }
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
            // ENV
            // [..., Nil]
            wasm.current_func
                .instructions()
                .ref_null(LISP_OBJ_HEAP_TYPE);

            for arg in args {
                compile(arg, wasm, env);
            }

            let n_args = args.len() as u32;
            wasm.current_func
                .instructions()
                .array_new_fixed(LISP_ARRAY_TYPE_IDX, n_args);

            compile(expr, wasm, env);
        }
        Expr::Let(bindings, body) => {
            todo!()
        }
        Expr::Prog(expressions) => {
            let len = expressions.len();
            if len == 0 {
                // Se o programa for vazio, retornamos Nil para não deixar a pilha vazia
                wasm.current_func
                    .instructions()
                    .ref_null(LISP_OBJ_HEAP_TYPE)
                    .struct_new(LISP_OBJ_TYPE_IDX);
            } else {
                for (i, expression) in expressions.iter().enumerate() {
                    compile(expression, wasm, env);

                    // Limpa a pilha para todas as expressões exceto a última
                    if i < len - 1 {
                        wasm.current_func.instructions().drop();
                    }
                }
            }
        }
        Expr::List(items) => todo!(),
    }
}

fn find_free_vars(node: &Expr, bound_in_params: &[String]) -> HashSet<String> {
    let mut free_vars = HashSet::new();
    let mut current_bound = bound_in_params.iter().cloned().collect::<HashSet<String>>();

    collect_free_vars(node, &mut current_bound, &mut free_vars);
    free_vars
}

fn collect_free_vars(node: &Expr, bound: &mut HashSet<String>, free: &mut HashSet<String>) {
    match node {
        Expr::Symbol(name) => {
            if !bound.contains(name) {
                free.insert(name.clone());
            }
        }
        Expr::Lambda(params, body) => {
            let mut new_bound = bound.clone();
            for p in params {
                new_bound.insert(p.clone());
            }
            collect_free_vars(body, bound, free);
        }
        _ => {}
    }
}

struct WasmCode {
    module: Module,
    current_func: Function,
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
    // start: StartSection,
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

// Lisp Array
const LISP_ARRAY_TYPE_IDX: u32 = 5;

// Function types
const LISP_FUNC_SIG_TYPE_IDX: u32 = 6;
const LISP_FUNC_SIG_LOCAL_ENV: u32 = 0;
const LISP_FUNC_SIG_LOCAL_ARGS: u32 = 1;

const MAIN_FUNC_SIG_TYPE_IDX: u32 = 7;

// Function INDEXES:
const MAIN_FUNC_IDX: u32 = 0;
const ADD_FUNC_IDX: u32 = 0;

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

        // Naming functions
        let mut function_names = NameMap::new();
        function_names.append(MAIN_FUNC_IDX, "main");
        function_names.append(ADD_FUNC_IDX, "+");
        // function_names.append(1, "-");
        // function_names.append(2, "*");
        // function_names.append(3, "/");
        names.functions(&function_names);

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

        // defining Lisp_array
        types
            .ty()
            .array(&StorageType::Val(LISP_OBJ_VAL_TYPE), false);

        // Defining Lisp Func Signature
        types.ty().func_type(&FuncType::new(
            [
                LISP_OBJ_VAL_TYPE,
                ValType::Ref(RefType {
                    nullable: false,
                    heap_type: HeapType::Concrete(LISP_ARRAY_TYPE_IDX),
                }),
            ],
            [LISP_OBJ_VAL_TYPE],
        ));

        // Defining Main Func Signature
        types
            .ty()
            .func_type(&FuncType::new([], [LISP_OBJ_VAL_TYPE]));

        // Naming types
        let mut type_names = NameMap::new();
        type_names.append(LISP_OBJ_TYPE_IDX, "lisp_obj");
        type_names.append(CLOSURE_TYPE_IDX, "closure");
        type_names.append(CONS_CELL_TYPE_IDX, "cons_cell");
        type_names.append(INTEGER_TYPE_IDX, "integer");
        type_names.append(FLOAT_TYPE_IDX, "float");
        type_names.append(LISP_ARRAY_TYPE_IDX, "lisp_array");
        type_names.append(LISP_FUNC_SIG_TYPE_IDX, "lisp_func_sig");
        type_names.append(MAIN_FUNC_SIG_TYPE_IDX, "main_func_sig");
        names.types(&type_names);

        // Define functions

        // Main func
        // functions.function(MAIN_FUNC_SIG_TYPE_IDX);
        let mut main_function = Function::new([]);
        // main_function.instructions();
        //     .ref_null(LISP_OBJ_HEAP_TYPE)
        //     .i32_const(30)
        //     .struct_new(INTEGER_TYPE_IDX)
        //     .i32_const(24)
        //     .struct_new(INTEGER_TYPE_IDX)
        //     .ref_null(LISP_OBJ_HEAP_TYPE)
        //     .struct_new(CONS_CELL_TYPE_IDX)
        //     .struct_new(CONS_CELL_TYPE_IDX)
        //     .call(0)
        //     .ref_cast_non_null(INTEGER_HEAP_TYPE)
        //     .struct_get(INTEGER_TYPE_IDX, 0)
        // .end();
        // code.function(&main_function);

        // + func
        functions.function(LISP_FUNC_SIG_TYPE_IDX);
        let mut builtin_function_plus = Function::new([]);
        builtin_function_plus
            .instructions()
            // Pega o car
            // .local_get(LISP_FUNC_SIG_LOCAL_ARGS)
            // .ref_cast_non_null(CONS_CELL_HEAP_TYPE)
            // .struct_get(CONS_CELL_TYPE_IDX, CONS_CELL_FIELD_CAR_IDX)
            // // pega o inteiro dentro do car
            // .ref_cast_non_null(INTEGER_HEAP_TYPE)
            // .struct_get(INTEGER_TYPE_IDX, 0)
            // // pega o cdr
            // .local_get(LISP_FUNC_SIG_LOCAL_ARGS)
            // .ref_cast_non_null(CONS_CELL_HEAP_TYPE)
            // .struct_get(CONS_CELL_TYPE_IDX, CONS_CELL_FIELD_CDR_IDX)
            // // pega o car do cdr
            // .ref_cast_non_null(CONS_CELL_HEAP_TYPE)
            // .struct_get(CONS_CELL_TYPE_IDX, CONS_CELL_FIELD_CAR_IDX)
            // // o outro inteiro do outro car
            // .ref_cast_non_null(INTEGER_HEAP_TYPE)
            // .struct_get(INTEGER_TYPE_IDX, 0)
            // // Soma os dois inteiros da pilha
            // .i32_add()
            // // Empacota o inteiro resultante da operação em uma nova box
            // .struct_new(INTEGER_TYPE_IDX)
            // Finaliza o código da função
            .local_get(LISP_FUNC_SIG_LOCAL_ARGS)
            .i32_const(0)
            .array_get(LISP_ARRAY_TYPE_IDX)
            .ref_cast_non_null(INTEGER_HEAP_TYPE)
            .struct_get(INTEGER_TYPE_IDX, 0)
            .local_get(LISP_FUNC_SIG_LOCAL_ARGS)
            .i32_const(1)
            .array_get(LISP_ARRAY_TYPE_IDX)
            .ref_cast_non_null(INTEGER_HEAP_TYPE)
            .struct_get(INTEGER_TYPE_IDX, 0)
            .i32_add()
            .struct_new(INTEGER_TYPE_IDX)
            .end();
        code.function(&builtin_function_plus);

        // let start = StartSection {
        //     function_index: 0,
        // };

        let mut exports = ExportSection::new();
        // exports.export("main", ExportKind::Func, MAIN_FUNC_IDX);

        Self {
            module,
            current_func: main_function,
            sections: WasmCodeSections {
                types,
                functions,
                tables,
                globals,
                code,
                names,
                imports,
                // start,
                exports,
            },
        }
    }

    fn finish(mut self) -> Vec<u8> {
        // Last function is the main function
        self.current_func.instructions().end();

        self.sections.functions.function(MAIN_FUNC_SIG_TYPE_IDX);
        self.sections.code.function(&self.current_func);

        let function_index = self.sections.functions.len() - 1;
        let start = StartSection { function_index };

        self.sections
            .exports
            .export("main", ExportKind::Func, function_index);

        self.module
            .section(&self.sections.types)
            .section(&self.sections.imports)
            .section(&self.sections.functions)
            .section(&self.sections.tables)
            .section(&self.sections.globals)
            .section(&self.sections.exports)
            // .section(&start)
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
// struct Environment {
//     scopes: Vec<HashMap<String, u32>>,
// }

#[derive(Clone, Debug)]
pub enum Binding {
    Global(u32),          // Índice na seção Global do Wasm
    Argument(u32),        // Índice no Array de Argumentos (local.get 1)
    Captured(u32),        // Índice no Struct de Ambiente (local.get 0)
}

pub struct Scope {
    pub bindings: HashMap<String, Binding>,
    pub env_struct_idx: Option<u32>, // Se for uma Lambda, qual o ID do seu Struct de Env?
}

pub struct Environment {
    pub scopes: Vec<Scope>,
}

impl Environment {
    fn new() -> Self {
        Self {
            // intialize with the top level scope
            scopes: vec![Scope{bindings: HashMap::new(), env_struct_idx: None}],
        }
    }

    fn scope_level(&self) -> u32 {
        (self.scopes.len() - 1) as u32
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope{bindings: HashMap::new(), env_struct_idx: None});
    }

    fn pop_scope(&mut self) {
        if self.scope_level() != 0 {
            self.scopes.pop();
        }
    }

    fn define(&mut self, name: String, index: u32) -> u32 {
        let idx = (self.scopes.len() + 1) as u32;

        if let Some(scope) = self.scopes.last_mut() {
            scope.bindings.insert(name, Binding::Argument(index));
        } else {
            panic!("define free")
        }

        idx
    }

    // fn resolve(&mut self, name: &str) -> Option<(u32, u32)> {
    //     for (i, scope) in self.scopes.iter().rev().enumerate() {
    //         if let Some(var) = scope.bindings.get(name) {
    //             return Some((*var, i as u32));
    //         }
    //     }
    //     None
    // }

    pub fn resolve(&self, name: &str) -> Option<Binding> {
        // Busca do escopo mais interno para o mais externo
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.bindings.get(name) {
                return Some(binding.clone());
            }
        }
        None
    }
}
