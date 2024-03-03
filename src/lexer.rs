use crate::token::{self, Token};
use crate::Res;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct Lexer {
    buffer: String,
    pos: usize,
    row: usize,
    col: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            buffer: "".to_string(),
            pos: 0,
            row: 0,
            col: 0,
        }
    }
    pub fn load_bytes(&mut self, s: String) {
        self.buffer = s;
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
    fn next_ch(&mut self) -> Option<u8> {
        self.pos += 1;
        let c = self.ch();
        if let Some(c) = c {
            if matches!(c, b'\n' | b'\x0C' | b'\r') {
                self.col = 0;
                self.row += 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    /// self.pos -> first unread char
    pub fn next_tok(&mut self) -> Res<Token> {
        self.skip_while(is_whitespace);
        let (init_pos, init_col, init_row) = (self.pos, self.col, self.row);
        let tok_kind = match self.ch() {
            None => token::EOF,
            Some(b'(') => {
                self.next_ch();
                token::PAREN_L
            }
            Some(b')') => {
                self.next_ch();
                token::PAREN_R
            }
            Some(b'&') => {
                self.next_ch();
                token::AND
            }
            Some(b'|') => {
                self.next_ch();
                token::OR
            }
            Some(b'!') => {
                self.next_ch();
                token::NOT
            }
            Some(b'=') => match self.next_ch() {
                Some(b'>') => {
                    self.next_ch();
                    token::IMPL
                }
                _ => {
                    self.skip_while(is_not_alphanumeric_whitespace);
                    token::INVALID
                }
            },
            Some(b'<') => match self.next_ch() {
                Some(b'=') => match self.next_ch() {
                    Some(b'>') => {
                        self.next_ch();
                        token::EQUIV
                    }
                    _ => {
                        self.skip_while(is_not_alphanumeric_whitespace);
                        token::INVALID
                    }
                },
                _ => {
                    self.skip_while(is_not_alphanumeric_whitespace);
                    token::INVALID
                }
            },
            Some(b'a'..=b'z' | b'A'..=b'Z') => {
                self.skip_while(is_alphanumeric);
                token::IDENT
            }
            _ => {
                self.skip_while(is_not_alphanumeric_whitespace);
                token::INVALID
            }
        };
        let s = &self.buffer[init_pos..self.pos];
        if tok_kind == token::INVALID {
            Err(InvalidTokenErr::new(s, init_row, init_col))
        } else {
            Ok(Token::new(tok_kind, s, init_row, init_col))
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
            self.next_ch();
        }
    }
}

fn is_alphanumeric(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn is_whitespace(c: u8) -> bool {
    c.is_ascii_whitespace()
}

fn is_not_alphanumeric_whitespace(c: u8) -> bool {
    !is_whitespace(c) && !is_alphanumeric(c)
}

#[derive(Debug)]
struct InvalidTokenErr {
    tok: String,
    row: usize,
    col: usize,
}

impl fmt::Display for InvalidTokenErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid token [{}:{}]: {}", self.row, self.col, self.tok)
    }
}

impl<'a> Error for InvalidTokenErr {}

impl<'a> InvalidTokenErr {
    fn new(s: &str, row: usize, col: usize) -> Box<Self> {
        Box::new(InvalidTokenErr {
            tok: s.to_string(),
            col,
            row,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{InvalidTokenErr, Lexer};
    use crate::token::{self, Token};
    use crate::Res;

    fn compare(lex: &mut Lexer, expected: &[Res<token::Token>]) {
        for exp in expected {
            // @lazy
            let e = format!("{:?}", exp);
            let l = format!("{:?}", lex.next_tok());
            if e != l {
                panic!("exptected=`{e}`, got=`{l}`")
            }
        }
    }

    #[test]
    fn test_tokens_and_errors() {
        let buffer = "x => y
x| y
x & y
x <=>y
!x
x&y
(x | y) & z
is_al_num <=> Is_Al_NuM
<<=> y
x <y
^
".to_string();
        let expected: &[Res<token::Token>] = &[
            Ok(Token::new(token::IDENT, "x", 1, 1)),
            Ok(Token::new(token::IMPL, "=>", 1, 3)),
            Ok(Token::new(token::IDENT, "y", 1, 6)),
            Ok(Token::new(token::IDENT, "x", 2, 1)),
            Ok(Token::new(token::OR, "|", 2, 2)),
            Ok(Token::new(token::IDENT, "y", 2, 4)),
            Ok(Token::new(token::IDENT, "x", 3, 1)),
            Ok(Token::new(token::AND, "&", 3, 3)),
            Ok(Token::new(token::IDENT, "y", 3, 5)),
            Ok(Token::new(token::IDENT, "x", 4, 1)),
            Ok(Token::new(token::EQUIV, "<=>", 4, 3)),
            Ok(Token::new(token::IDENT, "y", 4, 6)),
            Ok(Token::new(token::NOT, "!", 5, 1)),
            Ok(Token::new(token::IDENT, "x", 5, 2)),
            Ok(Token::new(token::IDENT, "x", 6, 1)),
            Ok(Token::new(token::AND, "&", 6, 2)),
            Ok(Token::new(token::IDENT, "y", 6, 3)),
            Ok(Token::new(token::PAREN_L, "(", 7, 1)),
            Ok(Token::new(token::IDENT, "x", 7, 2)),
            Ok(Token::new(token::OR, "|", 7, 4)),
            Ok(Token::new(token::IDENT, "y", 7, 6)),
            Ok(Token::new(token::PAREN_R, ")", 7, 7)),
            Ok(Token::new(token::AND, "&", 7, 9)),
            Ok(Token::new(token::IDENT, "z", 7, 11)),
            Ok(Token::new(token::IDENT, "is_al_num", 8, 1)),
            Ok(Token::new(token::EQUIV, "<=>", 8, 11)),
            Ok(Token::new(token::IDENT, "Is_Al_NuM", 8, 15)),
            Err(InvalidTokenErr::new("<<=>", 9, 1)),
            Ok(Token::new(token::IDENT, "y", 9, 6)),
            Ok(Token::new(token::IDENT, "x", 10, 1)),
            Err(InvalidTokenErr::new("<", 10, 3)),
            Ok(Token::new(token::IDENT, "y", 10, 4)),
            Err(InvalidTokenErr::new("^", 11, 1)),
            Ok(Token::new(token::EOF, "", 12, 0)),
        ];
        let mut lex = Lexer::new();
        lex.load_bytes(buffer);
        compare(&mut lex, expected);
    }
}
