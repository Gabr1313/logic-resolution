use std::io::{self, Write};

use crate::lexer;
use crate::parser;
use crate::token;
use crate::Res;

const PROMPT: &str = ">>";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    let lex = lexer::Lexer::new();
    let mut pars = parser::Parser::new(lex)?;
    print!("{} ", PROMPT);
    let _ = io::stdout().flush();
    for line in stdin.lines() {
        pars.load_bytes(line?)?;
        while pars.curr_tok().kind != token::Kind::Eof {
            match pars.parse_statement() {
                Ok(parsed) => println!("{}", parsed),
                Err(err) => println!("{}", err),
            }
        }

        print!("{} ", PROMPT);
        io::stdout().flush()?;
    }
    Ok(())
}
