use std::io::{self, Write};

use crate::lexer;
use crate::token;
use crate::Res;

const PROMPT: &str = ">>";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    print!("{} ", PROMPT);
    let _ = io::stdout().flush();
    let mut lex = lexer::Lexer::new();
    for line in stdin.lines() {
        let line = line?;
        lex.load_bytes(line);

        loop {
            match lex.next_tok() {
                Ok(token::Token {
                    kind: token::EOF, ..
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
