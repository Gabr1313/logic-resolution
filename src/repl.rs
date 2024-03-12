use crate::ast::Statement;
use crate::clause::SetClauses;
use crate::context;
use crate::error::Res;
use crate::lexer;
use crate::parser;
use std::io::{self, Write};

const PROMPT: &str = ">> ";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    let lex = lexer::Lexer::new();
    let mut pars = parser::Parser::new(lex)?;
    let mut context = context::Context::new();
    print!("{}", PROMPT);
    let _ = io::stdout().flush();
    for line in stdin.lines() {
        if let Err(err) = pars.load_bytes(line?) {
            eprintln!("{}", err);
        } else {
            loop {
                match pars.parse_statement_update_context(&mut context) {
                    Ok(Statement::Eof) => break,
                    Ok(Statement::Delete(n)) => println!("Statement {n} removed."),
                    Ok(Statement::Query) => println!("{}", context),
                    Ok(Statement::Execute) => {
                        let mut to_solve = SetClauses::from(&context);
                        if to_solve.find_box() {
                            println!("Box found:\n{}\n{}", context, to_solve.trace_from_box());
                        } else {
                            println!("Box not found.");
                        }
                    }
                    Ok(Statement::Formula(formula)) => println!("{}", formula),
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
        print!("{}", PROMPT);
        io::stdout().flush()?;
    }
    Ok(())
}
