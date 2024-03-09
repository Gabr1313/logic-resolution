use crate::error::{Res, ParseErr};
use crate::{ast, lexer, token};

#[derive(Debug)]
pub struct Parser {
    lex: lexer::Lexer,
    curr_tok: Option<token::Token>,
    peek_tok: Option<token::Token>,
}

impl Parser {
    pub fn new(lex: lexer::Lexer) -> Res<Parser> {
        let mut p = Parser {
            lex,
            curr_tok: None,
            peek_tok: None,
        };
        p.init()?;
        Ok(p)
    }

    fn init(&mut self) -> Res<()> {
        self.curr_tok = self.peek_tok.take();
        self.peek_tok = Some(self.lex.next_tok()?);
        self.curr_tok = self.peek_tok.take();
        self.peek_tok = Some(self.lex.next_tok()?);
        Ok(())
    }

    pub fn load_bytes(&mut self, buffer: String) -> Res<()> {
        self.lex.load_bytes(buffer);
        self.init()
    }

    fn curr_tok(&self) -> &token::Token {
        // should never panic because None is only the initial value
        self.curr_tok.as_ref().unwrap()
    }

    /// returns the previous token
    fn skip_tok(&mut self) -> Res<token::Token> {
        // should never panic because None is only the initial value
        let tok = self.curr_tok.take().unwrap();
        self.curr_tok = self.peek_tok.take();
        self.peek_tok = Some(self.lex.next_tok()?);
        Ok(tok)
    }

    /// it skips only the first token if it is invalid
    /// it does not skip what is there instead of `;`
    pub fn parse_formula(&mut self) -> Res<ast::Formula> {
        if self.curr_tok().kind() == token::Kind::Eof {
            return Ok(ast::Formula::Eof);
        }
        let stat = self.recursive_pratt(0)?;
        if self.curr_tok().kind() != token::Kind::SemiColon {
            return Err(ParseErr::new(
                self.curr_tok().clone(),
                format!("expected `{}`", token::Kind::SemiColon),
            ));
        }
        self.skip_tok()?;
        Ok(stat)
    }

    /// it does skip the first token if it is invalid
    fn recursive_pratt(&mut self, precedence: usize) -> Res<ast::Formula> {
        // pre: unary operator
        let mut formula = match self.curr_tok().kind() {
            token::Kind::Not => self.parse_unary(),
            token::Kind::ParenL => self.parse_paren(),
            token::Kind::Identifier => self.parse_leaf(),
            _ => {
                return Err(ParseErr::new(
                    self.skip_tok()?,
                    "not the beginning of a formula".to_string(),
                ))
            }
        };

        // post: binary operator
        while precedence < self.curr_tok().precedence() {
            match self.curr_tok().kind() {
                token::Kind::And | token::Kind::Or | token::Kind::Implies | token::Kind::Equiv => {
                    formula = self.parse_binary(formula?)
                }
                _ => break,
            }
        }
        formula
    }

    /// it does not skip what is there instead of `)`
    fn parse_paren(&mut self) -> Res<ast::Formula> {
        let paren_l = self.skip_tok()?;
        let f = self.recursive_pratt(paren_l.precedence())?;
        if self.curr_tok().kind() != token::Kind::ParenR {
            return Err(ParseErr::new(
                self.curr_tok().clone(),
                format!("expected `{}`", token::Kind::ParenR),
            ));
        }
        self.skip_tok()?;
        Ok(f)
    }

    fn parse_leaf(&mut self) -> Res<ast::Formula> {
        let t = self.skip_tok()?;
        Ok(ast::Formula::new_leaf(t.destory()))
    }

    fn parse_unary(&mut self) -> Res<ast::Formula> {
        let operator = self.skip_tok()?.kind();
        let p = operator.precedence();
        Ok(ast::Formula::new_unary(operator, self.recursive_pratt(p)?))
    }

    fn parse_binary(&mut self, left: ast::Formula) -> Res<ast::Formula> {
        let operator = self.skip_tok()?.kind();
        let p = operator.precedence();
        Ok(ast::Formula::new_binary(
            left,
            operator,
            self.recursive_pratt(p)?,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::Parser;
    use crate::lexer;
    use crate::token;

    #[test]
    fn test_parser() {
        let buffer = "
x;
!x;
x => y;
x | y;
x | y | z;
((x | y) | z);
(x | (y | z));
x & y;
x <=> y;
((x | y)) & z;
x <=> y => z | w & !v;
!x & y | z => w <=> v;
!x | (y | z) <=> !w => v & b;
=>
(x | y;
";
        // i suppose that the lexer tests pass
        let mut lex_test = lexer::Lexer::new();
        lex_test.load_bytes(buffer.to_string());
        let mut tokens = Vec::new();
        while let Ok(t) = lex_test.next_tok() {
            if t.kind() == token::Kind::Eof {
                break;
            }
            tokens.push(Some(t));
        }

        let expected: &[&str] = &[
            "x",
            "(!x)",
            "(x => y)",
            "(x | y)",
            "((x | y) | z)",
            "((x | y) | z)",
            "(x | (y | z))",
            "(x & y)",
            "(x <=> y)",
            "((x | y) & z)",
            "(x <=> (y => (z | (w & (!v)))))",
            "(((((!x) & y) | z) => w) <=> v)",
            "(((!x) | (y | z)) <=> ((!w) => (v & b)))",
            "Parse error [15:1]: got=`=>` (Implies): not the beginning of a formula",
            "Parse error [16:7]: got=`;` (SemiColon): expected `)`",
            "Parse error [16:7]: got=`;` (SemiColon): not the beginning of a formula",
            "EOF",
        ];
        let mut lex = lexer::Lexer::new();
        lex.load_bytes(buffer.to_string());
        let mut pars = Parser::new(lex).unwrap();

        for &exp in expected {
            let l = match pars.parse_formula() {
                Ok(s) => format!("{s}"),
                Err(s) => format!("{s}"),
            };
            if exp != l {
                panic!("expected=`{exp}`\ngot     =`{l}`")
            }
        }
    }
}
