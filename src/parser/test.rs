use super::Parser;
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
        "Parse error [16:1]: got=`=>` (Implies): not the beginning of a formula",
        "Parse error [17:7]: got=`;` (SemiColon): expected `)`",
        "Parse error [17:7]: got=`;` (SemiColon): not the beginning of a formula",
        "EOF",
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
        let l = match pars.parse_formula() {
            Ok(s) => format!("{s}"),
            Err(s) => format!("{s}"),
        };
        if exp != l {
            panic!("expected=`{exp}`\ngot     =`{l}`")
        }
    }
}
