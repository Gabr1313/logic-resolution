use crate::token;
use crate::error::{Res, InvalidTokenErr};

#[derive(Debug)]
pub struct Lexer {
    buffer: String,
    pos: usize,
    row: usize,
    col: usize,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            buffer: "".to_string(),
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
            Some(b'!') => {
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
                // TODO: save Identifier as a progressive number ?
                // (compare string O(n_id), trie(s.len()) -> to the easiest)
                self.skip_while(is_alphanumeric);
                token::Kind::Identifier
            }
            _ => {
                self.skip_while(is_not_whitespace);
                token::Kind::Invalid
            }
        };
        let s = self.buffer[init_pos..self.pos].to_string();
        if tok_kind == token::Kind::Invalid {
            Err(InvalidTokenErr::new(s, init_row, init_col))
        } else {
            Ok(token::Token::new(tok_kind, s, init_row, init_col))
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

#[cfg(test)]
mod test {
    use super::Lexer;
    use crate::token;
    use crate::error::{Res, InvalidTokenErr};

    fn compare(lex: &mut Lexer, expected: &[Res<token::Token>]) {
        for exp in expected {
            let e = format!("{:?}", exp);
            let l = format!("{:?}", lex.next_tok());
            if e != l {
                panic!("exptected=`{e}`, got=`{l}`")
            }
        }
    }

    #[test]
    fn test_lexer() {
        let buffer = "
x => y
x| y;
x & y
x <=>y   ;
!x ;
x&y
(x | y) & z
is_al_num <=> Is_Al_NuM
<<=> y
x <y
^
";
        let expected: &[Res<token::Token>] = &[
            Ok(token::Token::new(
                token::Kind::Identifier,
                "x".to_string(),
                2,
                1,
            )),
            Ok(token::Token::new(
                token::Kind::Implies,
                "=>".to_string(),
                2,
                3,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "y".to_string(),
                2,
                6,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "x".to_string(),
                3,
                1,
            )),
            Ok(token::Token::new(token::Kind::Or, "|".to_string(), 3, 2)),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "y".to_string(),
                3,
                4,
            )),
            Ok(token::Token::new(
                token::Kind::SemiColon,
                ";".to_string(),
                3,
                5,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "x".to_string(),
                4,
                1,
            )),
            Ok(token::Token::new(token::Kind::And, "&".to_string(), 4, 3)),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "y".to_string(),
                4,
                5,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "x".to_string(),
                5,
                1,
            )),
            Ok(token::Token::new(
                token::Kind::Equiv,
                "<=>".to_string(),
                5,
                3,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "y".to_string(),
                5,
                6,
            )),
            Ok(token::Token::new(
                token::Kind::SemiColon,
                ";".to_string(),
                5,
                10,
            )),
            Ok(token::Token::new(token::Kind::Not, "!".to_string(), 6, 1)),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "x".to_string(),
                6,
                2,
            )),
            Ok(token::Token::new(
                token::Kind::SemiColon,
                ";".to_string(),
                6,
                4,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "x".to_string(),
                7,
                1,
            )),
            Ok(token::Token::new(token::Kind::And, "&".to_string(), 7, 2)),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "y".to_string(),
                7,
                3,
            )),
            Ok(token::Token::new(
                token::Kind::ParenL,
                "(".to_string(),
                8,
                1,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "x".to_string(),
                8,
                2,
            )),
            Ok(token::Token::new(token::Kind::Or, "|".to_string(), 8, 4)),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "y".to_string(),
                8,
                6,
            )),
            Ok(token::Token::new(
                token::Kind::ParenR,
                ")".to_string(),
                8,
                7,
            )),
            Ok(token::Token::new(token::Kind::And, "&".to_string(), 8, 9)),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "z".to_string(),
                8,
                11,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "is_al_num".to_string(),
                9,
                1,
            )),
            Ok(token::Token::new(
                token::Kind::Equiv,
                "<=>".to_string(),
                9,
                11,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "Is_Al_NuM".to_string(),
                9,
                15,
            )),
            Err(InvalidTokenErr::new("<<=>".to_string(), 10, 1)),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "y".to_string(),
                10,
                6,
            )),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "x".to_string(),
                11,
                1,
            )),
            Err(InvalidTokenErr::new("<".to_string(), 11, 3)),
            Ok(token::Token::new(
                token::Kind::Identifier,
                "y".to_string(),
                11,
                4,
            )),
            Err(InvalidTokenErr::new("^".to_string(), 12, 1)),
            Ok(token::Token::new(token::Kind::Eof, "".to_string(), 13, 1)),
        ];
        let mut lex = Lexer::new();
        lex.load_bytes(buffer.to_string());
        compare(&mut lex, expected);
    }
}
