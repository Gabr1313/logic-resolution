use std::error::Error;
pub type Res<T> = std::result::Result<T, Box<dyn Error>>;

pub mod lexer;
pub mod token;
pub mod repl;
