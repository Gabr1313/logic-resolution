use crate::ast;
use crate::context::Context;
use crate::token;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::rc::{Rc, Weak};

#[cfg(test)]
mod test;

#[derive(Hash, Ord, PartialOrd, Clone, Debug)]
/// Equal is implementented using ptr
pub enum Atom {
    Positive(Rc<str>),
    Negative(Rc<str>),
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        if let (Atom::Positive(a), Atom::Positive(b)) = (self, other) {
            a.as_ptr() == b.as_ptr()
        } else if let (Atom::Negative(a), Atom::Negative(b)) = (self, other) {
            a.as_ptr() == b.as_ptr()
        } else {
            false
        }
    }
}
impl Eq for Atom {}

impl Atom {
    pub fn new_affermative(s: Rc<str>) -> Atom {
        Atom::Positive(s)
    }
    pub fn new_negative(s: Rc<str>) -> Atom {
        Atom::Negative(s)
    }
    pub fn opposite(&self) -> Atom {
        match self {
            Atom::Positive(x) => Atom::Negative(Rc::clone(x)),
            Atom::Negative(x) => Atom::Positive(Rc::clone(x)),
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
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
#[derive(Default, Clone, Debug)]
pub struct SetClauses {
    // using a BTreeSet i should avoid duplicates
    bt: BTreeMap<Rc<Clause>, Option<(Weak<Clause>, Weak<Clause>)>>,
}

impl From<&Context> for SetClauses {
    fn from(c: &Context) -> SetClauses {
        let bt = c
            .inner()
            .iter()
            .map(|x| x.set_clauses())
            .fold(BTreeMap::new(), |mut acc, x| {
                for x in x.as_ref().bt.iter() {
                    acc.insert(Rc::clone(x.0), x.1.clone()); // it uses Weak::clone() inside
                }
                acc
            });
        SetClauses { bt }
    }
}

impl From<Vec<SetClauses>> for SetClauses {
    fn from(clauses: Vec<SetClauses>) -> SetClauses {
        SetClauses {
            bt: clauses.into_iter().fold(BTreeMap::new(), |mut acc, x| {
                acc.extend(x.bt);
                acc
            }),
        }
    }
}

impl From<&ast::Formula> for SetClauses {
    fn from(formula: &ast::Formula) -> SetClauses {
        let mut c = SetClauses {
            bt: BTreeMap::new(),
        };
        c.append_formula(formula);
        c
    }
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
    fn append_formula(&mut self, formula: &ast::Formula) {
        // find `or` recursively than call append_to_clause()
        match formula {
            ast::Formula::Binary(x) => {
                let (left, operator, right) = x.parts();
                if operator == token::Kind::Or {
                    let mut bt = Clause::new();
                    // compiler does not evaluate the second expression if the first one is false
                    if SetClauses::append_atom(&mut bt, left)
                        && SetClauses::append_atom(&mut bt, right)
                    {
                        self.bt.insert(Rc::new(bt), None);
                    }
                } else {
                    debug_assert!(operator == token::Kind::And);
                    self.append_formula(left);
                    self.append_formula(right);
                }
            }
            ast::Formula::Unary(_) | ast::Formula::Leaf(_) => {
                let mut bt = Clause::new();
                if SetClauses::append_atom(&mut bt, formula) {
                    self.bt.insert(Rc::new(bt), None);
                }
            }
        };
    }

    fn append_atom(bt: &mut Clause, f: &ast::Formula) -> bool {
        match f {
            ast::Formula::Binary(x) => {
                let (left, operator, right) = x.parts();
                debug_assert!(operator == token::Kind::Or);
                // compiler does not evaluate the second expression if the first one is false
                SetClauses::append_atom(bt, left) && SetClauses::append_atom(bt, right)
            }
            ast::Formula::Unary(x) => {
                debug_assert!(x.operator() == token::Kind::Not);
                if let ast::Formula::Leaf(x) = x.right() {
                    // pruning: it is useless to have a clause like {!x, x, ...}
                    if bt.c.contains(&Atom::Positive(x.string())) {
                        // comparing 2 atoms is not slow because it uses
                        // pointers of the inner Rc value
                        false
                    } else {
                        bt.c.insert(Atom::Negative(x.string()));
                        true
                    }
                } else {
                    panic!("this should be a leaf, see ast::Formula::digest()");
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
        }
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

    // @todo? Horn... Nah, I don't think i will
    pub fn find_box(&mut self) -> bool {
        self.prune();
        let mut previous_len = 0;
        while previous_len != self.bt.len() {
            previous_len = self.bt.len();
            if self.square() {
                return true;
            }
        }
        false
    }

    /// returns true if box if found
    fn square(&mut self) -> bool {
        let mut new_clauses = SetClauses::default();
        let mut found = false;
        for (i, c1) in self.bt.iter().enumerate() {
            // is skip efficient? magic...
            for c2 in self.bt.iter().skip(i) {
                found |= new_clauses.extend_solve(Rc::clone(c1.0), Rc::clone(c2.0), self);
                if found {
                    break;
                }
            }
        }
        self.bt.extend(new_clauses.bt);
        found
    }

    /// returns true if box if found
    fn extend_solve(&mut self, c1: Rc<Clause>, c2: Rc<Clause>, parent: &SetClauses) -> bool {
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
                    Some(_) => return false,
                    None => pp = Some((atom, opposite)),
                }
            }
        }
        let (atom, opposite) = match pp {
            Some(x) => x,
            None => return false,
        };
        let new_clause: Clause =
            c1.c.iter()
                .filter(|x| *x != atom)
                .chain(c2.c.iter().filter(|x| *x != &opposite))
                .map(|x| x.clone()) // it uses Rc::clone() inside
                .collect::<BTreeSet<Atom>>()
                .into();

        let len = new_clause.c.len();
        // suddenly extend substitutes new values to old ones
        if !parent.bt.contains_key(&new_clause) {
            self.bt.insert(
                Rc::new(new_clause),
                Some((Rc::downgrade(&c1), Rc::downgrade(&c2))),
            );
        }
        len == 0
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
