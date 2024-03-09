use crate::{lexer, parser::Parser, token};

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

    for &exp in expected {
        // i suppose that the parser tests pass
        let l = match pars.parse_formula().unwrap().digest() {
            Ok(s) => format!("{s}"),
            Err(s) => format!("{s}"),
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

    let mut pars = Parser::new(lex).unwrap();
    for &exp in expected {
        // i suppose that the parser tests pass
        let l = match pars.parse_formula().unwrap().distribute() {
            Ok(s) => format!("{s}"),
            Err(s) => format!("{s}"),
        };
        if exp != l {
            panic!("expected=`{exp}`\ngot     =`{l}`")
        }
    }
}
