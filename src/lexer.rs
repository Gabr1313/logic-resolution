use crate::error::{InvalidTokenErr, Res};
use crate::token;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct Lexer {
    buffer: String,
    // @perf save ids as a progressive number ?
    //       -> problem: printing becomes a shitty: where to store the HashMap(Id, String)?
    ids: HashMap<String, Rc<String>>,
    pos: usize,
    row: usize,
    col: usize,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            buffer: "".to_string(),
            ids: HashMap::new(),
            pos: 0,
            row: 0,
            col: 0,
        }
    }
    pub fn load_bytes(&mut self, buffer: String) {
        self.buffer = buffer;
        self.pos = 0;
        self.row += 1;
        self.col = 1;
    }
    fn ch(&self) -> Option<u8> {
        if self.pos < self.buffer.len() {
            Some(self.buffer.as_bytes()[self.pos])
        } else {
            None
        }
    }
    fn skip_ch(&mut self) -> Option<u8> {
        if let Some(c) = self.ch() {
            if matches!(c, b'\n' | b'\x0C' | b'\r') {
                self.col = 1;
                self.row += 1;
            } else {
                self.col += 1;
            }
        }
        self.pos += 1;
        self.ch()
    }

    /// self.pos -> first unread char
    pub fn next_tok(&mut self) -> Res<token::Token> {
        self.skip_while(is_whitespace);
        let (init_pos, init_col, init_row) = (self.pos, self.col, self.row);
        let tok_kind = match self.ch() {
            None => token::Kind::Eof,
            Some(b';') => {
                self.skip_ch();
                token::Kind::SemiColon
            }
            Some(b'(') => {
                self.skip_ch();
                token::Kind::ParenL
            }
            Some(b')') => {
                self.skip_ch();
                token::Kind::ParenR
            }
            Some(b'&') => {
                self.skip_ch();
                token::Kind::And
            }
            Some(b'|') => {
                self.skip_ch();
                token::Kind::Or
            }
            Some(b'~') => {
                self.skip_ch();
                token::Kind::Not
            }
            Some(b'=') => match self.skip_ch() {
                Some(b'>') => {
                    self.skip_ch();
                    token::Kind::Implies
                }
                _ => {
                    self.skip_while(is_not_alphanumeric_whitespace);
                    token::Kind::Invalid
                }
            },
            Some(b'<') => match self.skip_ch() {
                Some(b'=') => match self.skip_ch() {
                    Some(b'>') => {
                        self.skip_ch();
                        token::Kind::Equiv
                    }
                    _ => {
                        self.skip_while(is_not_alphanumeric_whitespace);
                        token::Kind::Invalid
                    }
                },
                _ => {
                    self.skip_while(is_not_alphanumeric_whitespace);
                    token::Kind::Invalid
                }
            },
            Some(b'a'..=b'z' | b'A'..=b'Z') => {
                self.skip_while(is_alphanumeric);
                token::Kind::Identifier
            }
            _ => {
                self.skip_while(is_not_whitespace);
                token::Kind::Invalid
            }
        };
        let s = &self.buffer[init_pos..self.pos];
        if tok_kind == token::Kind::Invalid {
            Err(InvalidTokenErr::new(s.to_string(), init_row, init_col))
        } else {
            Ok(if let Some(rc) = self.ids.get(s) {
                token::Token::new(tok_kind, Rc::clone(rc), init_row, init_col)
            } else {
                let s = s.to_string();
                let rc = Rc::new(s.clone());
                self.ids.insert(s, Rc::clone(&rc));
                token::Token::new(tok_kind, rc, init_row, init_col)
            })
        }
    }

    /// self.pos -> after f
    fn skip_while(&mut self, f: fn(u8) -> bool) {
        if let Some(c) = self.ch() {
            if !f(c) {
                return;
            }
        }
        while let Some(c) = self.ch() {
            if !f(c) {
                return;
            }
            self.skip_ch();
        }
    }
}

fn is_alphanumeric(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn is_whitespace(c: u8) -> bool {
    c.is_ascii_whitespace()
}

fn is_not_whitespace(c: u8) -> bool {
    !is_whitespace(c)
}

fn is_not_alphanumeric_whitespace(c: u8) -> bool {
    !is_whitespace(c) && !is_alphanumeric(c)
}
