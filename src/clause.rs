use crate::{ast, token};
use std::{collections::BTreeSet, fmt};

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
    bt: BTreeSet<BTreeSet<Atom>>, // using a BTreeSet i should avoid duplicates
}

impl fmt::Display for Clauses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        s.push('{');
        for v in &self.bt {
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
        if self.bt.len() > 0 {
            s.truncate(s.len() - 2);
        }
        s.push('}');
        write!(f, "{}", s,)
    }
}

impl Clauses {
    pub fn new(formula: ast::Formula) -> Clauses {
        let mut c = Clauses {
            bt: BTreeSet::new(),
        };
        c.add(formula);
        c
    }

    pub fn merge(clauses: Vec<Clauses>) -> Clauses {
        Clauses {
            bt: clauses.into_iter().fold(BTreeSet::new(), |mut acc, x| {
                acc.extend(x.bt);
                acc
            }),
        }
    }

    /// sorted and deduplicated
    fn add(&mut self, formula: ast::Formula) {
        // find `or` recursively than call append_to_clause()
        match formula {
            ast::Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                if operator == token::Kind::Or {
                    let mut bt = BTreeSet::new();
                    // compiler does not evaluate the second expression if the first one is false
                    if Clauses::append_to_clause(&mut bt, left)
                        && Clauses::append_to_clause(&mut bt, right)
                    {
                        self.bt.insert(bt);
                    }
                } else {
                    debug_assert!(operator == token::Kind::And);
                    self.add(left);
                    self.add(right);
                }
            }
            ast::Formula::Unary(_) | ast::Formula::Leaf(_) => {
                let mut bt = BTreeSet::new();
                if Clauses::append_to_clause(&mut bt, formula) {
                    self.bt.insert(bt);
                }
            }
            ast::Formula::Eof => todo!("impossible"),
        }
    }

    fn append_to_clause(bt: &mut BTreeSet<Atom>, f: ast::Formula) -> bool {
        match f {
            ast::Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                debug_assert!(operator == token::Kind::Or);
                // compiler does not evaluate the second expression if the first one is false
                Clauses::append_to_clause(bt, left) && Clauses::append_to_clause(bt, right)
            }
            ast::Formula::Unary(x) => {
                debug_assert!(x.operator() == token::Kind::Not);
                if let ast::Formula::Leaf(x) = x.right() {
                    // pruning
                    if bt.contains(&Atom::Positive(x.string())) {
                        false
                    } else {
                        bt.insert(Atom::Negative(x.string()));
                        true
                    }
                } else {
                    debug_assert!(false); // impossible: see ast::Formula::digest()
                    false
                }
            }
            ast::Formula::Leaf(x) => {
                // pruning
                if bt.contains(&Atom::Negative(x.string())) {
                    false
                } else {
                    bt.insert(Atom::Positive(x.string()));
                    true
                }
            }
            ast::Formula::Eof => todo!("impossible"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Clauses;
    use crate::{ast, lexer, parser, token};

    fn compare(pars: &mut parser::Parser, expected: &[&str]) {
        for &exp in expected {
            let c = Clauses::new(pars.parse_formula().unwrap().distribute().unwrap());
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
a | a;
a & a;
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
            "{{a}}",
            "{{a}}",
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

    fn compare_merge(pars: &mut parser::Parser, expected: &str) {
        let mut v = Vec::new();
        loop {
            match pars.parse_formula() {
                Ok(ast::Formula::Eof) => break,
                Ok(parsed) => {
                    let c = Clauses::new(parsed.distribute().unwrap());
                    v.push(c);
                }
                Err(err) => panic!("{err}"),
            }
        }
        let t = Clauses::merge(v);
        let s = t.to_string();
        if expected != s {
            panic!("expected=`{expected}`\ngot     =`{s}`")
        }
    }

    #[test]
    fn test_merge() {
        let buffer = "
a | a;
a & a;
a & b;
b & a;
a | b;
b | a;
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

        let expected = "{{a}, {a, b}, {b}}";
        let mut lex = lexer::Lexer::new();
        lex.load_bytes(buffer.to_string());
        let mut parser = parser::Parser::new(lex).unwrap();
        compare_merge(&mut parser, expected);
    }
}
