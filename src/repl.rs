use std::io::{self, Write};

use crate::ast;
use crate::clause;
use crate::lexer;
use crate::parser;
use crate::error::Res;

const PROMPT: &str = ">> ";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    let lex = lexer::Lexer::new();
    let mut pars = parser::Parser::new(lex)?;
    print!("{}", PROMPT);
    let _ = io::stdout().flush();
    // let mut clauses = Vec::new();
    for line in stdin.lines() {
        if let Err(err) = pars.load_bytes(line?) {
            eprintln!("{}", err);
        } else {
            loop {
                match pars.parse_formula() {
                    Ok(ast::Formula::Eof) => break,
                    Ok(parsed) => {
                        println!("--[tree]->       {}", parsed);
                        let dist = parsed.distribute()?;
                        println!("--[distribute]-> {}", dist);
                        let c = clause::Clauses::new(dist);
                        println!("--[clauses]->    {}", c);
                        // clauses.push(c);
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
