use crate::ast::Formula;
use crate::clause::SetClauses;
use crate::error::{IndexOutOfBound, Res};
use std::rc::Rc;

pub struct InnerContext {
    formula: Rc<Formula>,
    set_clauses: Rc<SetClauses>,
}

impl InnerContext {
    fn new(formula: Rc<Formula>) -> Res<InnerContext> {
        let dist = formula.as_ref().clone().distribute()?;
        let set_clauses = Rc::new((&dist).into());
        Ok(InnerContext {
            formula,
            set_clauses,
        })
    }
    pub fn formula(&self) -> Rc<Formula> {
        Rc::clone(&self.formula)
    }
    pub fn set_clauses(&self) -> Rc<SetClauses> {
        Rc::clone(&self.set_clauses)
    }
}

pub struct Context {
    inner: Vec<InnerContext>,
}

impl Context {
    pub fn new() -> Context {
        Context { inner: Vec::new() }
    }
    pub fn push(&mut self, formula: Rc<Formula>) -> Res<()> {
        self.inner.push(InnerContext::new(formula)?);
        Ok(())
    }
    pub fn remove(&mut self, index: usize) -> Res<InnerContext> {
        if index < self.inner.len() {
            Ok(self.inner.remove(index))
        } else {
            Err(IndexOutOfBound::new(format!(
                "{} >= {} (number of formulas)",
                index,
                self.inner.len()
            )))
        }
    }
    pub fn inner(&self) -> &Vec<InnerContext> {
        &self.inner
    }
    pub fn vec_str(&self) -> Vec<String> {
        self.inner
            .iter()
            .enumerate()
            .map(|(i, f)| {
                format!(
                    "{i}: {} -> {}",
                    f.formula.as_ref(),
                    f.set_clauses().as_ref()
                )
            })
            .collect()
    }
}
