use crate::ast;
use crate::error::{InternalError, Res};
use crate::token;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::rc::{Rc, Weak};

#[cfg(test)]
mod test;

#[derive(Eq, Hash, PartialEq, Ord, PartialOrd, Clone)]
pub enum Atom {
    Positive(Rc<String>),
    Negative(Rc<String>),
}

impl Atom {
    pub fn new_affermative(s: Rc<String>) -> Atom {
        Atom::Positive(s)
    }
    pub fn new_negative(s: Rc<String>) -> Atom {
        Atom::Negative(s)
    }
    pub fn opposite(&self) -> Atom {
        match self {
            Atom::Positive(x) => Atom::Negative(Rc::clone(x)),
            Atom::Negative(x) => Atom::Positive(Rc::clone(x)),
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Clause {
    c: BTreeSet<Atom>,
}

impl Clause {
    fn new() -> Clause {
        Clause { c: BTreeSet::new() }
    }
}

impl From<BTreeSet<Atom>> for Clause {
    fn from(c: BTreeSet<Atom>) -> Clause {
        Clause { c }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .c
            .iter()
            .map(|x| match x {
                Atom::Positive(x) => format!("{x}"),
                Atom::Negative(x) => format!("{}{x}", token::Kind::Not),
            })
            .reduce(|acc, s| format!("{acc}, {s}"))
            .unwrap_or_default();
        write!(f, "{{{s}}}",)
    }
}

/// sort: positive before negative, then lexological order
#[derive(Default, Clone)]
pub struct SetClauses {
    // @perf would it be better to use HashSets?
    //       -> problem: print in tests would not be deterministic
    // using a BTreeSet i should avoid duplicates
    bt: BTreeMap<Rc<Clause>, Option<(Weak<Clause>, Weak<Clause>)>>,
}

impl fmt::Display for SetClauses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .bt
            .iter()
            .map(|x| x.0.to_string())
            .reduce(|acc, s| format!("{acc}, {s}"))
            .unwrap_or_default();
        write!(f, "{{{s}}}",)
    }
}

impl SetClauses {
    pub fn new(formula: ast::Formula) -> Res<SetClauses> {
        let mut c = SetClauses {
            bt: BTreeMap::new(),
        };
        c.append_formula(formula)?;
        Ok(c)
    }

    pub fn merge(clauses: Vec<SetClauses>) -> SetClauses {
        SetClauses {
            bt: clauses.into_iter().fold(BTreeMap::new(), |mut acc, x| {
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
                    let mut bt = Clause::new();
                    // compiler does not evaluate the second expression if the first one is false
                    if SetClauses::append_atom(&mut bt, left)?
                        && SetClauses::append_atom(&mut bt, right)?
                    {
                        self.bt.insert(Rc::new(bt), None);
                    }
                } else {
                    debug_assert!(operator == token::Kind::And);
                    self.append_formula(left)?;
                    self.append_formula(right)?;
                }
            }
            ast::Formula::Unary(_) | ast::Formula::Leaf(_) => {
                let mut bt = Clause::new();
                if SetClauses::append_atom(&mut bt, formula)? {
                    self.bt.insert(Rc::new(bt), None);
                }
            }
        };
        Ok(())
    }

    fn append_atom(bt: &mut Clause, f: ast::Formula) -> Res<bool> {
        Ok(match f {
            ast::Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                debug_assert!(operator == token::Kind::Or);
                // compiler does not evaluate the second expression if the first one is false
                SetClauses::append_atom(bt, left)? && SetClauses::append_atom(bt, right)?
            }
            ast::Formula::Unary(x) => {
                debug_assert!(x.operator() == token::Kind::Not);
                if let ast::Formula::Leaf(x) = x.right() {
                    // @perf comparing 2 atoms is slow: it compares inner values of Rc (String)
                    // pruning: it is useless to have a clause like {!x, x, ...}
                    if bt.c.contains(&Atom::Positive(x.string())) {
                        false
                    } else {
                        bt.c.insert(Atom::Negative(x.string()));
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
                if bt.c.contains(&Atom::Negative(x.string())) {
                    false
                } else {
                    bt.c.insert(Atom::Positive(x.string()));
                    true
                }
            }
        })
    }

    fn prune(&mut self) {
        let mut hm = HashMap::new();
        for (v, _) in &self.bt {
            for w in &v.as_ref().c {
                match w {
                    Atom::Positive(x) => hm
                        .entry(Rc::clone(x))
                        .and_modify(|(b, _): &mut (bool, bool)| *b = true)
                        .or_insert((true, false)),
                    Atom::Negative(x) => hm
                        .entry(Rc::clone(x))
                        .and_modify(|(_, b): &mut (bool, bool)| *b = true)
                        .or_insert((false, true)),
                };
            }
        }
        self.bt.retain(|v, _| {
            for w in &v.as_ref().c {
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

    // @todo? Horn
    pub fn find_box(&mut self) -> bool {
        self.prune();
        let empty = Clause::new();
        let mut previous_len = 0;
        while previous_len != self.bt.len() {
            previous_len = self.bt.len();
            self.square();
            if self.bt.contains_key(&empty) {
                return true;
            }
        }
        false
    }

    // @todo break when box is found
    fn square(&mut self) {
        let mut new_clauses = SetClauses::default();
        for (i, c1) in self.bt.iter().enumerate() {
            // is skip efficient? magic...
            for c2 in self.bt.iter().skip(i) {
                new_clauses.extend_solve(Rc::clone(c1.0), Rc::clone(c2.0), self);
            }
        }
        self.bt.extend(new_clauses.bt);
    }

    // @todo break when box is found
    fn extend_solve(&mut self, c1: Rc<Clause>, c2: Rc<Clause>, parent: &SetClauses) {
        let (c1, c2) = if c1.c.len() < c2.c.len() {
            (c1, c2)
        } else {
            (c2, c1)
        };
        let mut pp = None;
        for atom in &c1.as_ref().c {
            let opposite = atom.opposite();
            if c2.c.contains(&opposite) {
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
        let new_clause: Clause =
            c1.c.iter()
                .filter(|x| *x != atom)
                .chain(c2.c.iter().filter(|x| *x != &opposite))
                .map(|x| x.clone())
                .collect::<BTreeSet<Atom>>()
                .into();

        if !parent.bt.contains_key(&new_clause) {
            self.bt.insert(
                Rc::new(new_clause),
                Some((Rc::downgrade(&c1), Rc::downgrade(&c2))),
            );
        }
    }

    pub fn trace_from_box(&self) -> String {
        self.trace_from_box_vec()
            .into_iter()
            .reduce(|acc, s| format!("{acc}\n{s}"))
            .unwrap_or_default()
    }

    fn trace_from_box_vec(&self) -> Vec<String> {
        let mut trace = vec![];
        let empty = Rc::new(Clause::new());
        if !self.bt.contains_key(&empty) {
            return trace;
        }
        self.trace_from(Rc::downgrade(&empty), &mut trace);
        trace
    }

    fn trace_from(&self, clause: Weak<Clause>, trace: &mut Vec<String>) {
        if let Some((c1, c2)) = self.bt.get(&clause.upgrade().unwrap()).unwrap() {
            self.trace_from(Weak::clone(&c1), trace);
            self.trace_from(Weak::clone(&c2), trace);
            trace.push(format!(
                "{}, {} -> {}",
                c1.upgrade()
                    .expect("self.find_box() is poorly written")
                    .as_ref(),
                c2.upgrade()
                    .expect("self.find_box() is poorly written")
                    .as_ref(),
                clause
                    .upgrade()
                    .expect("self.find_box() is poorly written")
                    .as_ref()
            ))
        }
    }
}
