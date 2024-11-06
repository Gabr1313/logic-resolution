use super::Parser;
use crate::{context, slice_to_str};

#[test]
fn test_parser() {
    let buffer = "
x;
~x
x => y;
x | y
x | y | z
((x | y) | z);
(x | (y | z));
x & y
x <=> y;
((x | y)) & z;
x <=> y => z | w & ~v;
~x & y | z => w <=> v
~x | (y | z) <=> ~w => v & b
~(a&b&d);
!
a!
=>
(x | y;
exit;
help;
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
        "EXECUTE",
        "Parse error [17:2]: got=`!` (Bang): expected `SEPARATOR`",
        "EXECUTE",
        "Parse error [18:1]: got=`=>` (Implies): not the beginning of a formula",
        "Parse error [19:7]: got=`;` (Separator): expected `)`",
        "EXIT",
        "HELP",
        "END OF INPUT",
    ];

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
~y
0 => ~1
?
-0
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
        "QUERY
0: x -> {{x}}
1: (~y) -> {{~y}}
2: (x => (~(~y))) -> {{y, ~x}}",
        "DELETE 0
0: (~y) -> {{~y}}
1: (x => (~(~y))) -> {{y, ~x}}",
        "DELETE 1
0: (~y) -> {{~y}}",
    ];

    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = context::Context::new();

    for &exp in expected {
        let l = match pars.parse_statement_update_context(&mut context) {
            Ok(s) => format!("{s}\n{}", slice_to_str(&context.vec_str())),
            Err(s) => format!("{s}\n{}", slice_to_str(&context.vec_str())),
        };
        if exp != l {
            panic!("expected=`{exp}`\ngot     =`{l}`")
        }
    }
}
