use std::io::{self, Write};

use crate::ast;
use crate::lexer;
use crate::parser;
use crate::Res;

const PROMPT: &str = ">> ";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    let lex = lexer::Lexer::new();
    let mut pars = parser::Parser::new(lex)?;
    print!("{}", PROMPT);
    let _ = io::stdout().flush();
    for line in stdin.lines() {
        pars.load_bytes(line?)?;
        loop {
            match pars.parse_formula() {
                Ok(ast::Formula::Eof) => break,
                Ok(parsed) => println!("{}", parsed),
                Err(err) => println!("{}", err),
            }
        }

        print!("{}", PROMPT);
        io::stdout().flush()?;
    }
    Ok(())
}
