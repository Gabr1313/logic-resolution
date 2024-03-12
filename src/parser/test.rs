use super::Parser;
use crate::context;
use crate::lexer;
use crate::token;

#[test]
fn test_parser() {
    let buffer = "
x;
~x;
0 <=> ~1;
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
        "(x <=> (~(~x)))",
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
        "Parse error [18:2]: got=`!` (Bang): expected `;`",
        "Execute",
        "Parse error [19:1]: got=`=>` (Implies): not the beginning of a formula",
        "Parse error [20:7]: got=`;` (SemiColon): expected `)`",
        "Parse error [20:7]: got=`;` (SemiColon): not the beginning of a formula",
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
        let l = match pars.parse_statement_update_context(&mut context) {
            Ok(s) => format!("{s}"),
            Err(s) => format!("{s}"),
        };
        if exp != l {
            panic!("expected=`{exp}`\ngot     =`{l}`")
        }
    }
}
