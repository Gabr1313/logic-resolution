use crate::ast::Statement;
use crate::clause::SetClauses;
use crate::context;
use crate::error::Res;
use crate::help;
use crate::parser;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};

const PROMPT: &str = ">> ";
pub const SPACES: &str = " ";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    let mut pars = parser::Parser::new()?;
    let mut context = context::Context::new();
    println!("Type `help`");
    print!("{}", PROMPT);
    io::stdout().flush()?;
    for line in stdin.lines() {
        if eval_print(&mut pars, line?, &mut context, SPACES)? {
            io::stdout().flush()?; // do i need this here?
            break;
        }
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
    eval_print(&mut pars, buf, &mut context, "")?;
    Ok(())
}

/// returns true if exit is read
fn eval_print(
    pars: &mut parser::Parser,
    line: String,
    context: &mut context::Context,
    spaces: &str,
) -> Res<bool> {
    if let Err(err) = pars.load_bytes(line) {
        eprintln!("{}", err);
    } else {
        loop {
            match pars.parse_statement_update_context(context) {
                Ok(Statement::Eoi) => break,
                Ok(Statement::Exit) => return Ok(true),
                Ok(Statement::Help) => println!("{spaces}{}", help::help()),
                Ok(Statement::Delete(n)) => println!("{spaces}Formula {n} removed."),
                Ok(Statement::Query) => println!(
                    "{spaces}{}",
                    context
                        .vec_str()
                        .into_iter()
                        .reduce(|acc, s| format!("{acc}\n{spaces}{s}"))
                        .unwrap_or_default()
                ),
                Ok(Statement::Execute) => {
                    let mut to_solve = SetClauses::from(&*context);
                    if to_solve.find_box() {
                        println!("{spaces}Box found:");
                        println!(
                            "{spaces}{}",
                            context
                                .vec_str()
                                .into_iter()
                                .reduce(|acc, s| format!("{acc}\n{spaces}{s}"))
                                .unwrap_or_default()
                        );
                        println!(
                            "{spaces}{}",
                            to_solve
                                .trace_from_box()
                                .into_iter()
                                .reduce(|acc, s| format!("{acc}\n{spaces}{s}"))
                                .unwrap_or_default()
                        );
                    } else {
                        println!("{spaces}Box not found.");
                    }
                }
                Ok(Statement::Formula(formula)) => println!("{spaces}{}", formula),
                Err(err) => eprintln!("{spaces}{}", err),
            }
        }
    };
    Ok(false)
}
