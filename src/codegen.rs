// use crate::parser::Expr;

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

use std::fs;

use crate::parser::Expr;

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
