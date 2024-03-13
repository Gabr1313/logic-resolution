use crate::ast::Statement;
use crate::clause::SetClauses;
use crate::context;
use crate::error::Res;
use crate::parser;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};

const PROMPT: &str = ">> ";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    let mut pars = parser::Parser::new()?;
    let mut context = context::Context::new();
    print!("{}", PROMPT);
    let _ = io::stdout().flush();
    for line in stdin.lines() {
        eval_print(&mut pars, line?, &mut context)?;
        print!("{}", PROMPT);
        io::stdout().flush()?;
    }
    Ok(())
}

pub fn rep(filename: &str) -> Res<()> {
    let mut file = File::open(filename)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let mut pars = parser::Parser::new()?;
    let mut context = context::Context::new();
    eval_print(&mut pars, buf, &mut context)?;
    Ok(())
}

fn eval_print(pars: &mut parser::Parser, line: String, context: &mut context::Context) -> Res<()> {
    Ok(if let Err(err) = pars.load_bytes(line) {
        eprintln!("{}", err);
    } else {
        loop {
            match pars.parse_statement_update_context(context) {
                Ok(Statement::Eof) => break,
                Ok(Statement::Delete(n)) => println!("Formula {n} removed."),
                Ok(Statement::Query) => println!("{}", context),
                Ok(Statement::Execute) => {
                    let mut to_solve = SetClauses::from(&*context);
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
    })
}
