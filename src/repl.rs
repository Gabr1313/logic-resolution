use std::io::{self, Write};

use crate::lexer;
use crate::token;
use crate::Res;

const PROMPT: &str = ">>";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    let mut lex = lexer::Lexer::new();
    print!("{} ", PROMPT);
    let _ = io::stdout().flush();
    for line in stdin.lines() {
        lex.load_bytes(line?);
        loop {
            match lex.next_tok() {
                Ok(token::Token {
                    kind: token::Kind::Eof, ..
                }) => break,
                Ok(tok) => println!("{:?}", tok),
                Err(err) => println!("{:?}", err),
            }
        }

        print!("{} ", PROMPT);
        io::stdout().flush()?;
    }
    Ok(())
}
