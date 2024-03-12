use crate::{context, lexer, parser::Parser, token};

use super::Statement;

#[test]
fn test_digest() {
    let buffer = "
~x;
x & y;
x | y;
x => y;
x <=> y;
~x => ~y;
x <=> y => z;
x | y => z;
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
        "Execute",
        "Found end of file",
    ];

    // i suppose that the lexer tests pass
    let mut lex_test = lexer::Lexer::new();
    lex_test.load_bytes(buffer.to_string());
    let mut tokens = Vec::new();
    while let Ok(t) = lex_test.next_tok() {
        if t.kind() == token::Kind::Eof {
            break;
        }
        tokens.push(Some(t));
    }
    let mut lex = lexer::Lexer::new();
    lex.load_bytes(buffer.to_string());
    let mut pars = Parser::new(lex).unwrap();
    let mut context = context::Context::new();

    for &exp in expected {
        // i suppose that the parser tests pass
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
x <=> y;
a | (b & c & (d | e | (f & g)));
a & (b | c | (d & e & (f | g)));
";
    let expected: &[&str] = &[
        "(((x | (~x)) & (x | (~y))) & ((y | (~x)) & (y | (~y))))",
        "(((a | b) & (a | c)) & ((a | ((d | e) | f)) & (a | ((d | e) | g))))",
        "(a & ((((b | c) | d) & ((b | c) | e)) & ((b | c) | (f | g))))",
    ];

    // i suppose that the lexer tests pass
    let mut lex_test = lexer::Lexer::new();
    lex_test.load_bytes(buffer.to_string());
    let mut tokens = Vec::new();
    while let Ok(t) = lex_test.next_tok() {
        if t.kind() == token::Kind::Eof {
            break;
        }
        tokens.push(Some(t));
    }
    let mut lex = lexer::Lexer::new();
    lex.load_bytes(buffer.to_string());
    let mut context = context::Context::new();

    let mut pars = Parser::new(lex).unwrap();
    for &exp in expected {
        // i suppose that the parser tests pass
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
