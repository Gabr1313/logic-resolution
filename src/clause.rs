use std::{collections::HashSet, fmt};

use crate::{ast, token};

#[derive(Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Atom {
    Positive(String),
    Negative(String),
}

impl Atom {
    pub fn new_affermative(s: String) -> Atom {
        Atom::Positive(s)
    }
    pub fn new_negative(s: String) -> Atom {
        Atom::Negative(s)
    }
}

pub struct Clauses {
    vec: Vec<HashSet<Atom>>,
}

impl fmt::Display for Clauses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        s.push('{');
        for v in &self.vec {
            s.push('{');
            for w in v {
                match w {
                    Atom::Positive(x) => s.push_str(&x),
                    Atom::Negative(x) => {
                        s.push('!');
                        s.push_str(&x);
                    }
                }
                s.push_str(", ");
            }
            debug_assert!(v.len() > 0);
            s.truncate(s.len() - 2);
            s.push_str("}, ");
        }
        if self.vec.len() > 0 {
            s.truncate(s.len() - 2);
        }
        s.push('}');
        write!(f, "{}", s,)
    }
}

impl Clauses {
    pub fn new() -> Clauses {
        // TODO: ? with_capacity
        Clauses { vec: Vec::new() }
    }

    pub fn add(&mut self, formula: ast::Formula) {
        // find `or` recursively than call append_to_clause()
        match formula {
            ast::Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                if operator == token::Kind::Or {
                    let mut v = HashSet::new();
                    // compiler does not evaluate the second expression if the first one is false
                    if Clauses::append_to_clause(&mut v, left)
                        && Clauses::append_to_clause(&mut v, right)
                    {
                        self.vec.push(v);
                    }
                } else {
                    debug_assert!(operator == token::Kind::And);
                    self.add(left);
                    self.add(right);
                }
            }
            ast::Formula::Unary(_) | ast::Formula::Leaf(_) => {
                let mut v = HashSet::new();
                if Clauses::append_to_clause(&mut v, formula) {
                    self.vec.push(v);
                }
            }
            ast::Formula::Eof => todo!("impossible"),
        }
    }

    fn append_to_clause(hs: &mut HashSet<Atom>, f: ast::Formula) -> bool {
        match f {
            ast::Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                debug_assert!(operator == token::Kind::Or);
                // compiler does not evaluate the second expression if the first one is false
                Clauses::append_to_clause(hs, left) && Clauses::append_to_clause(hs, right)
            }
            ast::Formula::Unary(x) => {
                debug_assert!(x.operator() == token::Kind::Not);
                if let ast::Formula::Leaf(x) = x.right() {
                    // pruning
                    if hs.contains(&Atom::Positive(x.string())) {
                        false
                    } else {
                        hs.insert(Atom::Negative(x.string()));
                        true
                    }
                } else {
                    todo!("impossible: see pre_distribute");
                }
            }
            ast::Formula::Leaf(x) => {
                // pruning
                if hs.contains(&Atom::Negative(x.string())) {
                    false
                } else {
                    hs.insert(Atom::Positive(x.string()));
                    true
                }
            }
            ast::Formula::Eof => todo!("impossible"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Atom, Clauses};
    use crate::{lexer, parser, token};
    use std::fmt;

    struct FakeClauses {
        vec: Vec<Vec<Atom>>,
    }

    impl FakeClauses {
        fn from(c: Clauses) -> FakeClauses {
            let vec = c
                .vec
                .into_iter()
                .map(|e| {
                    let mut x: Vec<_> = e.into_iter().collect();
                    x.sort_unstable();
                    x
                })
                .collect();
            FakeClauses { vec }
        }
    }

    impl fmt::Display for FakeClauses {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut s = String::new();
            s.push('{');
            for v in &self.vec {
                s.push('{');
                for w in v {
                    match w {
                        Atom::Positive(x) => s.push_str(&x),
                        Atom::Negative(x) => {
                            s.push('!');
                            s.push_str(&x);
                        }
                    }
                    s.push_str(", ");
                }
                debug_assert!(v.len() > 0);
                s.truncate(s.len() - 2);
                s.push_str("}, ");
            }
            if self.vec.len() > 0 {
                s.truncate(s.len() - 2);
            }
            s.push('}');
            write!(f, "{}", s,)
        }
    }

    fn compare(pars: &mut parser::Parser, expected: &[&str]) {
        for &exp in expected {
            let mut c = Clauses::new();
            c.add(pars.parse_formula().unwrap().distribute().unwrap());
            let fc = FakeClauses::from(c);
            let s = fc.to_string();
            if exp != s {
                panic!("expected=`{exp}`\ngot     =`{s}`")
            }
        }
    }

    #[test]
    fn test_clauses() {
        let buffer = "
x;
!x;
a <=> b;
a | (b & c & (d | e | (f & g)));
a & (b | c | (d & e & (f | g)));
(b | ((a | !a) | (c | !d)));
        ";
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

        let expected: &[&str] = &[
            "{{x}}",
            "{{!x}}",
            "{{a, !b}, {b, !a}}",
            "{{a, b}, {a, c}, {a, d, e, f}, {a, d, e, g}}",
            "{{a}, {b, c, d}, {b, c, e}, {b, c, f, g}}",
            "{}",
        ];
        let mut lex = lexer::Lexer::new();
        lex.load_bytes(buffer.to_string());
        let mut parser = parser::Parser::new(lex).unwrap();
        compare(&mut parser, expected);
    }
}
