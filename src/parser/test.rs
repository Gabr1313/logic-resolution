use super::Parser;
use crate::context;
use crate::lexer;
use crate::token;

#[test]
fn test_parser() {
    let buffer = "
x;
~x;
x => y;
x | y;
x | y | z;
((x | y) | z);
(x | (y | z));
x & y;
x <=> y;
((x | y)) & z;
x <=> y => z | w & ~v;
~x & y | z => w <=> v;
~x | (y | z) <=> ~w => v & b;
~(a&b&d);
!
a!
=>
(x | y;
";
    let expected: &[&str] = &[
        "x",
        "(~x)",
        "(x => y)",
        "(x | y)",
        "((x | y) | z)",
        "((x | y) | z)",
        "(x | (y | z))",
        "(x & y)",
        "(x <=> y)",
        "((x | y) & z)",
        "(x <=> (y => (z | (w & (~v)))))",
        "(((((~x) & y) | z) => w) <=> v)",
        "(((~x) | (y | z)) <=> ((~w) => (v & b)))",
        "(~((a & b) & d))",
        "Execute",
        "Parse error [17:2]: got=`!` (Bang): expected `;`",
        "Execute",
        "Parse error [18:1]: got=`=>` (Implies): not the beginning of a formula",
        "Parse error [19:7]: got=`;` (SemiColon): expected `)`",
        "Parse error [19:7]: got=`;` (SemiColon): not the beginning of a formula",
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
    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = context::Context::new();

    for &exp in expected {
        let l = match pars.parse_statement(&mut context) {
            Ok(s) => format!("{s}"),
            Err(s) => format!("{s}"),
        };
        if exp != l {
            panic!("expected=`{exp}`\ngot     =`{l}`")
        }
    }
}

#[test]
fn test_parser_context() {
    let buffer = "
x;
~y;
0 => ~1;
?
-0;
-1;
";
    let expected: &[&str] = &[
        "x
0: x -> {{x}}",
        "(~y)
0: x -> {{x}}
1: (~y) -> {{~y}}",
        "(x => (~(~y)))
0: x -> {{x}}
1: (~y) -> {{~y}}
2: (x => (~(~y))) -> {{y, ~x}}",
        "Query
0: x -> {{x}}
1: (~y) -> {{~y}}
2: (x => (~(~y))) -> {{y, ~x}}",
        "Delete 0
0: (~y) -> {{~y}}
1: (x => (~(~y))) -> {{y, ~x}}",
        "Delete 1
0: (~y) -> {{~y}}",
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
    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = context::Context::new();

    for &exp in expected {
        let l = match pars.parse_statement_update_context(&mut context) {
            Ok(s) => format!("{s}\n{context}"),
            Err(s) => format!("{s}\n{context}"),
        };
        if exp != l {
            panic!("expected=`{exp}`\ngot     =`{l}`")
        }
    }
}
