use crate::{context, parser::Parser};

use super::Statement;

#[test]
fn test_digest() {
    let buffer = "
~x;
x & y
x | y;
x => y
x <=> y
~x => ~y;
x <=> y => z;
x | y => z;
(a & (~(a <=> b)));
!
";
    let expected: &[&str] = &[
        "(~x)",
        "(x & y)",
        "(x | y)",
        "((~x) | y)",
        "((x & y) | ((~x) & (~y)))",
        "(x | (~y))",
        "((x & ((~y) | z)) | ((~x) & (y & (~z))))",
        "(((~x) & (~y)) | z)",
        "(a & ((a & (~b)) | ((~a) & b)))",
        "EXECUTE",
        "END OF INPUT",
    ];

    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = context::Context::new();

    for &exp in expected {
        let parsed = pars.parse_statement_update_context(&mut context).unwrap();
        let l = if let Statement::Formula(f) = parsed {
            match f.digest() {
                Ok(s) => format!("{s}"),
                Err(s) => format!("{s}"),
            }
        } else {
            format!("{parsed}")
        };
        if exp != l {
            panic!("expected=`{exp}`\ngot     =`{l}`")
        }
    }
}

#[test]
fn test_distribute() {
    let buffer = "
x <=> y
a | (b & c & (d | e | (f & g)));
a & (b | c | (d & e & (f | g)))
";
    let expected: &[&str] = &[
        "(((x | (~x)) & (x | (~y))) & ((y | (~x)) & (y | (~y))))",
        "(((a | b) & (a | c)) & ((a | ((d | e) | f)) & (a | ((d | e) | g))))",
        "(a & ((((b | c) | d) & ((b | c) | e)) & ((b | c) | (f | g))))",
    ];

    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = context::Context::new();

    for &exp in expected {
        let parsed = pars.parse_statement_update_context(&mut context).unwrap();
        let l = if let Statement::Formula(f) = parsed {
            match f.distribute() {
                Ok(s) => format!("{s}"),
                Err(s) => format!("{s}"),
            }
        } else {
            format!("{parsed}")
        };
        if exp != l {
            panic!("expected=`{exp}`\ngot     =`{l}`")
        }
    }
}
