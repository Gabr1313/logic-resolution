use crate::clause;
use crate::clause::SetClauses;
use crate::error::Execute;
use crate::error::Feof;
use crate::error::Res;
use crate::lexer;
use crate::parser;
use std::io::{self, Write};

const PROMPT: &str = ">> ";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    let lex = lexer::Lexer::new();
    let mut pars = parser::Parser::new(lex)?;
    print!("{}", PROMPT);
    let _ = io::stdout().flush();
    let mut clauses = Vec::new();
    for line in stdin.lines() {
        if let Err(err) = pars.load_bytes(line?) {
            eprintln!("{}", err);
        } else {
            loop {
                match pars.parse_formula() {
                    Ok(parsed) => {
                        println!("--[tree]->    {}", parsed);
                        let dist = parsed.distribute()?;
                        // println!("--[distribute]-> {}", dist);
                        let c = clause::SetClauses::new(dist)?; // should not panic
                        println!("--[clauses]-> {}", c);
                        clauses.push(c);
                    }
                    Err(err) => {
                        // @todo better design: so ugly
                        if let Some(_) = err.downcast_ref::<Feof>() {
                            break;
                        }
                        if let Some(_) = err.downcast_ref::<Execute>() {
                            let mut to_solve = SetClauses::merge(clauses.clone());
                            if to_solve.find_box() {
                                println!("Box found:\n{}", to_solve.trace_from_box());
                            } else {
                                println!("Box not found."); // @todo print all the clauses found
                            }
                        } else {
                            eprintln!("{}", err);
                        }
                    }
                }
            }
        }
        print!("{}", PROMPT);
        io::stdout().flush()?;
    }
    Ok(())
}
