use super::Clauses;
use crate::{ast, lexer, parser, token};

#[test]
fn test_clauses() {
    let buffer = "
x;
~x;
a | a;
a & a;
a <=> b;
a | (b & c & (d | e | (f & g)));
a & (b | c | (d & e & (f | g)));
(b | ((a | ~a) | (c | ~d)));
";
    let expected: &[&str] = &[
        "{{x}}",
        "{{~x}}",
        "{{a}}",
        "{{a}}",
        "{{a, ~b}, {b, ~a}}",
        "{{a, b}, {a, c}, {a, d, e, f}, {a, d, e, g}}",
        "{{a}, {b, c, d}, {b, c, e}, {b, c, f, g}}",
        "{}",
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
    let mut pars = parser::Parser::new(lex).unwrap();

    for &exp in expected {
        let c = Clauses::new(pars.parse_formula().unwrap().distribute().unwrap());
        let s = match c {
            Ok(c) => c.to_string(),
            Err(c) => c.to_string(),
        };
        if exp != s {
            panic!("expected=`{exp}`\ngot     =`{s}`")
        }
    }
}

#[test]
fn test_merge() {
    let test = (
        "
a | a;
a & a;
a & b;
b & a;
a | b;
b | a;
",
        "{{a}, {a, b}, {b}}",
    );
    let (buffer, expected) = test;

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
    let mut pars = parser::Parser::new(lex).unwrap();

    let mut v = Vec::new();
    loop {
        match pars.parse_formula().unwrap() {
            ast::Formula::Eof => break,
            parsed => {
                let c = Clauses::new(parsed.distribute().unwrap()).unwrap();
                v.push(c);
            }
        }
    }
    let t = Clauses::merge(v);
    let s = t.to_string();
    if expected != s {
        panic!("expected=`{expected}`\ngot     =`{s}`")
    }
}

#[test]
fn test_prune() {
    let test = (" a | b; b | c; c | ~a; ~c | a; ", "{{a, ~c}, {c, ~a}}");
    let (buffer, expected) = test;

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
    let mut pars = parser::Parser::new(lex).unwrap();

    let mut v = Vec::new();
    loop {
        match pars.parse_formula().unwrap() {
            ast::Formula::Eof => break,
            parsed => {
                let c = Clauses::new(parsed.distribute().unwrap()).unwrap();
                v.push(c);
            }
        }
    }
    let mut t = Clauses::merge(v);
    t.prune();
    let s = t.to_string();
    if expected != s {
        panic!("expected=`{expected}`\ngot     =`{s}`")
    }
}

#[test]
fn test_find_box() {
    let tests = &[
        ("a;", false),
        ("a;~a;", true),
        ("(~B|C) & ~(A&~B) & (A|((B|C)&~C)); ~(A&B&C);", true),
        ("(~(B&C)) & (A=>(C<=>B)) & (~C=>A) & (~B|(A=>~C));", false),
    ];

    for (buf, exp) in tests {
        let mut lex_test = lexer::Lexer::new();
        lex_test.load_bytes(buf.to_string());
        let mut tokens = Vec::new();
        while let Ok(t) = lex_test.next_tok() {
            if t.kind() == token::Kind::Eof {
                break;
            }
            tokens.push(Some(t));
        }

        let mut lex = lexer::Lexer::new();
        lex.load_bytes(buf.to_string());
        let mut pars = parser::Parser::new(lex).unwrap();

        let mut v = Vec::new();
        loop {
            match pars.parse_formula().unwrap() {
                ast::Formula::Eof => break,
                parsed => {
                    let c = Clauses::new(parsed.distribute().unwrap()).unwrap();
                    v.push(c);
                }
            }
        }
        let mut t = Clauses::merge(v);
        if *exp != t.find_box() {
            panic!("expected=`{exp}`\ngot     =`{}`", !exp)
        }
    }
}
