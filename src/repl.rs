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
        // TODO: don't parse a line until it ends with `;`?
        if let Err(err) = pars.load_bytes(line?) {
            eprintln!("{}", err);
        } else {
            loop {
                match pars.parse_formula() {
                    Ok(ast::Formula::Eof) => break,
                    Ok(parsed) => {
                        println!("{} --[simplify]-> {}", parsed.clone(), parsed.distribute()?)
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
        print!("{}", PROMPT);
        io::stdout().flush()?;
    }
    Ok(())
}
