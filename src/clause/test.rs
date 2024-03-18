use super::SetClauses;
use crate::{ast::Statement, context::Context, parser::Parser, slice_to_str};

#[test]
fn test_clauses() {
    let buffer = "
x
~x
a | a;
a & a
a <=> b;
a | (b & c & (d | e | (f & g)));
a & (b | c | (d & e & (f | g)))
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
        "END OF INPUT",
    ];

    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = Context::new();

    for &exp in expected {
        let parsed = pars.parse_statement_update_context(&mut context).unwrap();
        let s = if let Statement::Formula(formula) = parsed {
            let c: SetClauses = (&formula.distribute().unwrap()).into();
            format!("{c}")
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
a & a
a & b
b & a
a | b;
b | a;
",
        "{{a}, {a, b}, {b}}",
    );
    let (buffer, expected) = test;

    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = Context::new();

    let mut v = Vec::new();
    loop {
        match pars.parse_statement_update_context(&mut context) {
            Ok(Statement::Eoi) => break,
            Ok(Statement::Formula(formula)) => {
                let c: SetClauses = (&formula.distribute().unwrap()).into();
                v.push(c);
            }
            Err(err) => panic!("{}", err),
            Ok(p) => panic!("{}", p),
        }
    }
    let t: SetClauses = v.into();
    let s = t.to_string();
    if expected != s {
        panic!("expected=`{expected}`\ngot     =`{s}`")
    }
}

#[test]
fn test_prune() {
    let test = (" a | b; b | c; c | ~a; ~c | a;", "{{a, ~c}, {c, ~a}}");
    let (buffer, expected) = test;

    let mut pars = Parser::new().unwrap();
    pars.load_bytes(buffer.to_string()).unwrap();
    let mut context = Context::new();

    let mut v = Vec::new();
    loop {
        match pars.parse_statement_update_context(&mut context) {
            Ok(Statement::Eoi) => break,
            Ok(Statement::Formula(formula)) => {
                let c: SetClauses = (&formula.distribute().unwrap()).into();
                v.push(c);
            }
            Err(err) => panic!("{}", err),
            Ok(p) => panic!("{}", p),
        }
    }
    let mut t: SetClauses = v.into();
    t.prune();
    let s = t.to_string();
    if expected != s {
        panic!("expected=`{expected}`\ngot     =`{s}`")
    }
}

#[test]
fn test_find_box() {
    let tests = &[
        ("a", false),
        ("a;~a", true),
        ("(~B|C) & ~(A&~B) & (A|((B|C)&~C)); ~(A&B&C)", true),
        ("(~(B&C)) & (A=>(C<=>B)) & (~C=>A) & (~B|(A=>~C));", false),
    ];

    for (buffer, exp) in tests {
        let mut pars = Parser::new().unwrap();
        pars.load_bytes(buffer.to_string()).unwrap();
        let mut context = Context::new();

        let mut v = Vec::new();
        loop {
            match pars.parse_statement_update_context(&mut context) {
                Ok(Statement::Eoi) => break,
                Ok(Statement::Formula(formula)) => {
                    let c: SetClauses = (&formula.distribute().unwrap()).into();
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
        let mut t: SetClauses = v.into();
        if *exp != t.find_box() {
            panic!("expected=`{exp}`\ngot     =`{}`", !exp)
        }
    }
}

#[test]
fn test_trace_from_box() {
    let tests = &[
        ("a;", vec![]),
        ("a;~a", vec!["{~a}, {a} -> {}"]),
        (
            "(~B|C) & ~(A&~B) & (A|(B|C)&~C); ~(A&B&C);",
            vec![
                "{B, ~A}, {~A, ~B, ~C} -> {~A, ~C}",
                "{C, ~B}, {B, ~A} -> {C, ~A}",
                "{~A, ~C}, {C, ~A} -> {~A}",
                "{C, ~B}, {A, B, C} -> {A, C}",
                "{A, ~C}, {A, C} -> {A}",
                "{~A}, {A} -> {}",
            ],
        ),
        ("(~(B&C)) & (A=>(C<=>B)) & (~C=>A) & (~B|(A=>~C));", vec![]),
        (
            "a; a <=> b; 0 & ~1;",
            vec!["{~a, ~b}, {b, ~a} -> {~a}", "{~a}, {a} -> {}"],
        ),
    ];

    for (buffer, exp) in tests {
        let mut pars = Parser::new().unwrap();
        pars.load_bytes(buffer.to_string()).unwrap();
        let mut context = Context::new();

        let mut v = Vec::new();
        loop {
            match pars.parse_statement_update_context(&mut context) {
                Ok(Statement::Eoi) => break,
                Ok(Statement::Formula(formula)) => {
                    let c: SetClauses = (&formula.distribute().unwrap()).into();
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
        let mut t: SetClauses = v.into();
        t.find_box();
        let trace = t.trace_from_box();
        if *exp != trace {
            panic!(
                "expected=`{}`\ngot     =`{}`",
                slice_to_str(exp, ""),
                slice_to_str(&trace, ""),
            )
        }
    }
}
