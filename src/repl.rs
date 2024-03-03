use std::io::{self, Write};
use std::rc::Rc;

use crate::lexer;
use crate::rc_substr::RcSubstr;
use crate::token;
use crate::Res;

const PROMPT: &str = ">>";

pub fn repl() -> Res<()> {
    let stdin = io::stdin();
    print!("{} ", PROMPT);
    let _ = io::stdout().flush();
    let mut lex = lexer::Lexer::new();
    for line in stdin.lines() {
        lex.load_bytes(RcSubstr::new(Rc::from(line?)));
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
