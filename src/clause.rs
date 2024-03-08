use std::fmt;

use crate::{ast, token};

pub enum Atom {
    Affermative(String),
    Negative(String),
}

impl Atom {
    pub fn new_affermative(s: String) -> Atom {
        Atom::Affermative(s)
    }
    pub fn new_negative(s: String) -> Atom {
        Atom::Negative(s)
    }
}

pub struct Clauses {
    vec: Vec<Vec<Atom>>,
}

impl fmt::Display for Clauses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        s.push('{');
        for v in &self.vec {
            s.push('{');
            for w in v {
                match w {
                    Atom::Affermative(x) => s.push_str(&x),
                    Atom::Negative(x) => {
                        s.push('!');
                        s.push_str(&x);
                    }
                }
                s.push_str(", ");
            }
            s.truncate(s.len() - 2);
            s.push_str("}, ");
        }
        s.truncate(s.len() - 2);
        s.push('}');
        write!(f, "{}", s,)
    }
}

impl Clauses {
    pub fn new() -> Clauses {
        // TODO: ? with_capacity
        Clauses { vec: Vec::new() }
    }

    pub fn add(&mut self, formula: &ast::Formula) {
        // find `or` recursively than call append_to_clause()
        match formula {
            ast::Formula::Binary(x) => {
                if x.operator() == token::Kind::Or {
                    let mut v = Vec::new();
                    Clauses::append_to_clause(&mut v, x.left());
                    Clauses::append_to_clause(&mut v, x.right());
                    self.vec.push(v);
                } else {
                    debug_assert!(x.operator() == token::Kind::And);
                    self.add(x.left());
                    self.add(x.right());
                }
            }
            ast::Formula::Unary(_) | ast::Formula::Leaf(_) => {
                let mut v = Vec::new();
                Clauses::append_to_clause(&mut v, formula);
                self.vec.push(v);
            }
            ast::Formula::Eof => todo!("impossible"),
        }
    }

    fn append_to_clause(v: &mut Vec<Atom>, f: &ast::Formula) {
        match f {
            ast::Formula::Binary(x) => {
                debug_assert!(x.operator() == token::Kind::Or);
                Clauses::append_to_clause(v, x.left());
                Clauses::append_to_clause(v, x.right());
            }
            ast::Formula::Unary(x) => {
                debug_assert!(x.operator() == token::Kind::Not);
                if let ast::Formula::Leaf(x) = x.right() {
                    v.push(Atom::Negative(x.string()));
                } else {
                    todo!("impossible: see pre_distribute");
                }
            }
            ast::Formula::Leaf(x) => v.push(Atom::Affermative(x.string())),
            ast::Formula::Eof => todo!("impossible"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Clauses;
    use crate::{lexer, parser, token};

    fn compare(pars: &mut parser::Parser, expected: &[&str]) {
        for &exp in expected {
            let mut c = Clauses::new();
            c.add(&pars.parse_formula().unwrap().distribute().unwrap());
            let s = c.to_string();
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
            "{{a, !a}, {a, !b}, {b, !a}, {b, !b}}",
            "{{a, b}, {a, c}, {a, d, e, f}, {a, d, e, g}}",
            "{{a}, {b, c, d}, {b, c, e}, {b, c, f, g}}",
        ];
        let mut lex = lexer::Lexer::new();
        lex.load_bytes(buffer.to_string());
        let mut parser = parser::Parser::new(lex).unwrap();
        compare(&mut parser, expected);
    }
}
