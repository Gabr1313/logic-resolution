use crate::ast;
use crate::error::{InternalError, InternalErrorTok, Res};
use crate::token;
use std::collections::{BTreeSet, HashMap};
use std::fmt;

#[cfg(test)]
mod test;

#[derive(Eq, Hash, PartialEq, Ord, PartialOrd, Clone)]
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
    /// @todo: so inefficient
    pub fn opposite(&self) -> Atom {
        match self {
            Atom::Positive(x) => Atom::Negative(x.to_string()),
            Atom::Negative(x) => Atom::Positive(x.to_string()),
        }
    }
}

/// sort: positive before negative, then lexological order
#[derive(Default)]
pub struct Clauses {
    // @pref would it be better to use HashSets? -> problem: print in tests
    //       would not be deterministic
    // using a BTreeSet i should avoid duplicates
    bt: BTreeSet<BTreeSet<Atom>>,
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
                        s.push_str(token::Kind::Not.as_str());
                        s.push_str(&x);
                    }
                }
                s.push_str(", ");
            }
            if v.len() > 0 {
                s.truncate(s.len() - 2);
            }
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
    pub fn new(formula: ast::Formula) -> Res<Clauses> {
        let mut c = Clauses {
            bt: BTreeSet::new(),
        };
        c.append_formula(formula)?;
        Ok(c)
    }

    pub fn merge(clauses: Vec<Clauses>) -> Clauses {
        Clauses {
            bt: clauses.into_iter().fold(BTreeSet::new(), |mut acc, x| {
                acc.extend(x.bt);
                acc
            }),
        }
    }

    fn append_formula(&mut self, formula: ast::Formula) -> Res<()> {
        // find `or` recursively than call append_to_clause()
        match formula {
            ast::Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                if operator == token::Kind::Or {
                    let mut bt = BTreeSet::new();
                    // compiler does not evaluate the second expression if the first one is false
                    if Clauses::append_atom(&mut bt, left)? && Clauses::append_atom(&mut bt, right)?
                    {
                        self.bt.insert(bt);
                    }
                } else {
                    debug_assert!(operator == token::Kind::And);
                    self.append_formula(left)?;
                    self.append_formula(right)?;
                }
            }
            ast::Formula::Unary(_) | ast::Formula::Leaf(_) => {
                let mut bt = BTreeSet::new();
                if Clauses::append_atom(&mut bt, formula)? {
                    self.bt.insert(bt);
                }
            }
            ast::Formula::Eof => {
                return Err(InternalErrorTok::new(
                    token::Kind::Eof,
                    "should not be in ast".to_string(),
                ))
            }
        };
        Ok(())
    }

    fn append_atom(bt: &mut BTreeSet<Atom>, f: ast::Formula) -> Res<bool> {
        Ok(match f {
            ast::Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                debug_assert!(operator == token::Kind::Or);
                // compiler does not evaluate the second expression if the first one is false
                Clauses::append_atom(bt, left)? && Clauses::append_atom(bt, right)?
            }
            ast::Formula::Unary(x) => {
                debug_assert!(x.operator() == token::Kind::Not);
                if let ast::Formula::Leaf(x) = x.right() {
                    // pruning: it is useless to have a clause like {!x, x, ...}
                    if bt.contains(&Atom::Positive(x.string())) {
                        false
                    } else {
                        bt.insert(Atom::Negative(x.string()));
                        true
                    }
                } else {
                    return Err(InternalError::new(
                        "this should be a leaf, see ast::Formula::digest()".to_string(),
                    ));
                }
            }
            ast::Formula::Leaf(x) => {
                // pruning: it is useless to have a clause like {!x, x, ...}
                if bt.contains(&Atom::Negative(x.string())) {
                    false
                } else {
                    bt.insert(Atom::Positive(x.string()));
                    true
                }
            }
            ast::Formula::Eof => {
                return Err(InternalErrorTok::new(
                    token::Kind::Eof,
                    "should not be in ast".to_string(),
                ))
            }
        })
    }

    fn prune(&mut self) {
        let mut hm = HashMap::new();
        for v in &self.bt {
            for w in v {
                match w {
                    Atom::Positive(x) => hm
                        .entry(x.clone()) // @todo not clone: Rc or use an ID instead of String
                        .and_modify(|(b, _): &mut (bool, bool)| *b = true)
                        .or_insert((true, false)),
                    Atom::Negative(x) => hm
                        .entry(x.clone()) // @todo not clone: Rc or use an ID instead of String
                        .and_modify(|(_, b): &mut (bool, bool)| *b = true)
                        .or_insert((false, true)),
                };
            }
        }
        self.bt.retain(|v| {
            for w in v {
                let b = match w {
                    Atom::Positive(x) => hm[x].1,
                    Atom::Negative(x) => hm[x].0,
                };
                if b == false {
                    return false;
                }
            }
            true
        });
    }

    // @todo Backtracking
    // @todo? Horn
    // @design would it be better to pass &self and self.clone() ?
    /// self.prune() is called here, it is unefficient to also call it before
    pub fn find_box(&mut self) -> bool {
        self.prune();
        let empty = BTreeSet::new();
        let mut previous_len = 0;
        while previous_len != self.bt.len() {
            previous_len = self.bt.len();
            self.square();
            if self.bt.contains(&empty) {
                return true;
            }
        }
        false
    }

    fn square(&mut self) {
        let mut new_clauses = Clauses::default();
        for (i, c1) in self.bt.iter().enumerate() {
            // is skip efficient? magic...
            for c2 in self.bt.iter().skip(i) {
                new_clauses.extend_solve(c1, c2);
            }
        }
        self.bt.extend(new_clauses.bt);
    }

    fn extend_solve(&mut self, c1: &BTreeSet<Atom>, c2: &BTreeSet<Atom>) {
        let (c1, c2) = if c1.len() < c2.len() {
            (c1, c2)
        } else {
            (c2, c1)
        };
        let mut pp = None;
        for atom in c1 {
            let opposite = atom.opposite();
            if c2.contains(&opposite) {
                match pp {
                    // pruning: it is useless to have a clause like {!x, x, ...}
                    Some(_) => return,
                    None => pp = Some((atom, opposite)),
                }
            }
        }
        let (atom, opposite) = match pp {
            Some(x) => x,
            None => return,
        };
        let new_clause = c1
            .iter()
            .filter(|x| *x != atom)
            .chain(c2.iter().filter(|x| *x != &opposite))
            .map(|x| x.clone())
            .collect();

        self.bt.insert(new_clause);
    }
}
