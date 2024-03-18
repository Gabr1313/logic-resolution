use core::fmt;

pub mod ast;
pub mod clause;
pub mod context;
pub mod error;
pub mod help;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod token;

pub fn slice_to_str(v: &[impl fmt::Display], prefix: &str) -> String {
    if v.len() == 0 {
        return "".to_string();
    }
    let first = format!("{prefix}{}", v.first().unwrap());
    v.iter()
        .skip(1)
        .fold(first, |acc, s| format!("{acc}\n{prefix}{s}"))
}
