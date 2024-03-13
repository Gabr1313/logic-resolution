use super::SetClauses;
use crate::{
    ast::{self, Statement},
    context, lexer,
    parser::Parser,
    token,
};

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
        "End of input",
    ];

    // i suppose that the lexer tests pass
    let mut lex_test = lexer::Lexer::new();
    lex_test.load_bytes(buffer.to_string());
    let mut tokens = Vec::new();
    while let Ok(t) = lex_test.next_tok() {
        if t.kind() == token::Kind::Eoi {
            break;
        }
        tokens.push(Some(t));
    }
    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = context::Context::new();

    for &exp in expected {
        let parsed = pars.parse_statement_update_context(&mut context).unwrap();
        let s = if let ast::Statement::Formula(f) = parsed {
            match SetClauses::new(&f.distribute().unwrap()) {
                Ok(s) => format!("{s}"),
                Err(s) => format!("{s}"),
            }
        } else {
            format!("{parsed}")
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
        if t.kind() == token::Kind::Eoi {
            break;
        }
        tokens.push(Some(t));
    }
    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = context::Context::new();

    let mut v = Vec::new();
    loop {
        match pars.parse_statement_update_context(&mut context) {
            Ok(Statement::Eoi) => break,
            Ok(Statement::Formula(formula)) => {
                let c = SetClauses::new(&formula.distribute().unwrap()).unwrap();
                v.push(c);
            }
            Err(err) => {
                panic!("{}", err);
            }
            Ok(p) => {
                panic!("{}", p);
            }
        }
    }
    let t = SetClauses::merge(v);
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
        if t.kind() == token::Kind::Eoi {
            break;
        }
        tokens.push(Some(t));
    }
    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = context::Context::new();

    let mut v = Vec::new();
    loop {
        match pars.parse_statement_update_context(&mut context) {
            Ok(Statement::Eoi) => break,
            Ok(Statement::Formula(formula)) => {
                let c = SetClauses::new(&formula.distribute().unwrap()).unwrap();
                v.push(c);
            }
            Err(err) => {
                panic!("{}", err);
            }
            Ok(p) => {
                panic!("{}", p);
            }
        }
    }
    let mut t = SetClauses::merge(v);
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

    for (buffer, exp) in tests {
        let mut lex_test = lexer::Lexer::new();
        lex_test.load_bytes(buffer.to_string());
        let mut tokens = Vec::new();
        while let Ok(t) = lex_test.next_tok() {
            if t.kind() == token::Kind::Eoi {
                break;
            }
            tokens.push(Some(t));
        }
        let mut pars = Parser::new().unwrap();
        pars.load_bytes(buffer.to_string()).unwrap();
        let mut context = context::Context::new();

        let mut v = Vec::new();
        loop {
            match pars.parse_statement_update_context(&mut context) {
                Ok(Statement::Eoi) => break,
                Ok(Statement::Formula(formula)) => {
                    let c = SetClauses::new(&formula.distribute().unwrap()).unwrap();
                    v.push(c);
                }
                Err(err) => {
                    panic!("{}", err);
                }
                Ok(p) => {
                    panic!("{}", p);
                }
            }
        }
        let mut t = SetClauses::merge(v);
        if *exp != t.find_box() {
            panic!("expected=`{exp}`\ngot     =`{}`", !exp)
        }
    }
}

#[test]
fn test_trace_from_box() {
    let tests = &[
        ("a;", ""),
        ("a;~a;", "{~a}, {a} -> {}"),
        (
            "(~B|C) & ~(A&~B) & (A|((B|C)&~C)); ~(A&B&C);",
            "{B, ~A}, {~A, ~B, ~C} -> {~A, ~C}
{C, ~B}, {B, ~A} -> {C, ~A}
{~A, ~C}, {C, ~A} -> {~A}
{C, ~B}, {A, B, C} -> {A, C}
{A, ~C}, {A, C} -> {A}
{~A}, {A} -> {}",
        ),
        ("(~(B&C)) & (A=>(C<=>B)) & (~C=>A) & (~B|(A=>~C));", ""),
        ("a; a <=> b; 0 & ~1;", "{~a, ~b}, {b, ~a} -> {~a}
{~a}, {a} -> {}"),
    ];

    for (buffer, exp) in tests {
        // i want to context to reset every time
        let mut lex_test = lexer::Lexer::new();
        lex_test.load_bytes(buffer.to_string());
        let mut tokens = Vec::new();
        while let Ok(t) = lex_test.next_tok() {
            if t.kind() == token::Kind::Eoi {
                break;
            }
            tokens.push(Some(t));
        }
        let mut pars = Parser::new().unwrap();
        pars.load_bytes(buffer.to_string()).unwrap();
        let mut context = context::Context::new();

        let mut v = Vec::new();
        loop {
            match pars.parse_statement_update_context(&mut context) {
                Ok(Statement::Eoi) => break,
                Ok(Statement::Formula(formula)) => {
                    let c = SetClauses::new(&formula.distribute().unwrap()).unwrap();
                    v.push(c);
                }
                Err(err) => {
                    panic!("{}", err);
                }
                Ok(p) => {
                    panic!("{}", p);
                }
            }
        }
        let mut t = SetClauses::merge(v);
        t.find_box();
        let trace = t.trace_from_box();
        if *exp != &trace {
            panic!("expected=`{exp}`\ngot     =`{}`", trace)
        }
    }
}
