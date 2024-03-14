use crate::error::{InvalidTokenErr, Res};
use crate::token;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct Lexer {
    buffer: String,
    ids: HashMap<String, Rc<str>>,
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
        self.skip_while(is_space);
        let (init_pos, init_col, init_row) = (self.pos, self.col, self.row);
        let tok_kind = match self.ch() {
            None => token::Kind::Eoi,
            Some(b';') | Some(b'\x0C') | Some(b'\r') | Some(b'\n') => {
                self.skip_ch();
                token::Kind::Separator
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
            Some(b'!') => {
                self.skip_ch();
                token::Kind::Bang
            }
            Some(b'?') => {
                self.skip_ch();
                token::Kind::Question
            }
            Some(b'-') => {
                self.skip_ch();
                token::Kind::Minus
            }
            Some(b'=') => match self.skip_ch() {
                Some(b'>') => {
                    self.skip_ch();
                    token::Kind::Implies
                }
                _ => {
                    self.skip_while(is_not_alphanumeric_space);
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
                        self.skip_while(is_not_alphanumeric_space);
                        token::Kind::Invalid
                    }
                },
                _ => {
                    self.skip_while(is_not_alphanumeric_space);
                    token::Kind::Invalid
                }
            },
            Some(b'0'..=b'9') => {
                self.skip_while(is_digit);
                token::Kind::Number
            }
            Some(b'a'..=b'z' | b'A'..=b'Z') => {
                self.skip_while(is_alphanumeric);
                token::Kind::Identifier
            }
            _ => {
                self.skip_ch();
                token::Kind::Invalid
            }
        };
        let s = &self.buffer[init_pos..self.pos];
        if tok_kind == token::Kind::Invalid {
            Err(InvalidTokenErr::new(s.to_string(), init_row, init_col))
        } else {
            // if I crate another instance of the string the comparison beetween
            // atoms does not work anymore
            Ok(if let Some(rc) = self.ids.get(s) {
                token::Token::new(tok_kind, Rc::clone(rc), init_row, init_col)
            } else {
                let rc = s.into();
                // up to now there are only 2 keyword, so I don't worry that much
                // an HashMap would be a good alternative
                match s {
                    "exit" => token::Token::new(token::Kind::Exit, rc, init_row, init_col),
                    "help" => token::Token::new(token::Kind::Help, rc, init_row, init_col),
                    _ => {
                        self.ids.insert(s.to_string(), Rc::clone(&rc));
                        token::Token::new(tok_kind, rc, init_row, init_col)
                    }
                }
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

fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn is_space(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

fn is_not_alphanumeric_space(c: u8) -> bool {
    !is_space(c) && !is_alphanumeric(c)
}
