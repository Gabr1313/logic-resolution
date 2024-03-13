use std::rc::Rc;

use crate::context::Context;
use crate::error::{ParseErr, Res};
use crate::{ast, lexer, token};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct Parser {
    lex: lexer::Lexer,
    curr_tok: Option<token::Token>,
    peek_tok: Option<token::Token>,
}

impl Parser {
    pub fn new() -> Res<Parser> {
        let mut p = Parser {
            lex: lexer::Lexer::new(),
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
        self.curr_tok
            .as_ref()
            .expect("None should be only the intial value")
    }

    /// returns the previous token
    fn skip_tok(&mut self) -> Res<token::Token> {
        let tok = self
            .curr_tok
            .take()
            .expect("None should be only the intial value");
        self.curr_tok = self.peek_tok.take();
        self.peek_tok = Some(self.lex.next_tok()?);
        Ok(tok)
    }

    pub fn parse_statement_update_context(&mut self, context: &mut Context) -> Res<ast::Statement> {
        let retval = self.parse_statement(context)?;
        match retval {
            ast::Statement::Delete(n) => {
                context.remove(n)?;
                Ok(retval)
            }
            ast::Statement::Formula(f) => {
                context.push(Rc::new(f.clone()))?; 
                Ok(f.into())
            }
            _ => Ok(retval),
        }
    }

    /// does NOT auto-update the context
    /// skips only the first token if it is invalid
    /// does not skip what is there instead of ``
    pub fn parse_statement(&mut self, context: &Context) -> Res<ast::Statement> {
        Ok(match self.curr_tok().kind() {
            token::Kind::Eof => ast::Statement::Eof,
            token::Kind::Bang => {
                self.skip_tok()?;
                ast::Statement::Execute
            }
            token::Kind::Question => {
                self.skip_tok()?;
                ast::Statement::Query
            }
            token::Kind::Minus => self.parse_delete()?,
            _ => {
                let stat = self.recursive_pratt(0, context)?;
                if self.curr_tok().kind() != token::Kind::SemiColon {
                    return Err(ParseErr::new(
                        self.curr_tok().clone(),
                        format!("expected `{}`", token::Kind::SemiColon),
                    ));
                }
                self.skip_tok()?;
                stat.into()
            }
        })
    }

    /// it does skip the first token if it is invalid
    fn recursive_pratt(&mut self, precedence: usize, context: &Context) -> Res<ast::Formula> {
        // pre: unary operator
        let mut formula = match self.curr_tok().kind() {
            token::Kind::Not => self.parse_unary(context),
            token::Kind::ParenL => self.parse_paren(context),
            token::Kind::Identifier => self.parse_leaf(),
            token::Kind::Number => self.parse_number(context),
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
                    formula = self.parse_binary(formula?, context)
                }
                _ => break,
            }
        }
        formula
    }

    /// it does not skip what is there instead of `)`
    fn parse_paren(&mut self, context: &Context) -> Res<ast::Formula> {
        let paren_l = self.skip_tok()?;
        let f = self.recursive_pratt(paren_l.precedence(), context)?;
        if self.curr_tok().kind() != token::Kind::ParenR {
            return Err(ParseErr::new(
                self.curr_tok().clone(),
                format!("expected `{}`", token::Kind::ParenR),
            ));
        }
        self.skip_tok()?;
        Ok(f)
    }

    fn parse_delete(&mut self) -> Res<ast::Statement> {
        self.skip_tok()?;
        if self.curr_tok().kind() == token::Kind::Number {
            let n = self.skip_tok()?.literal().as_ref().parse()?;
            if self.curr_tok().kind() == token::Kind::SemiColon {
                self.skip_tok()?;
                Ok(ast::Statement::Delete(n))
            } else {
                Err(ParseErr::new(
                    self.curr_tok().clone(),
                    format!("expected `{}`", token::Kind::SemiColon),
                ))
            }
        } else {
            Err(ParseErr::new(
                self.curr_tok().clone(),
                format!("expected `{}`", token::Kind::Number),
            ))
        }
    }

    fn parse_leaf(&mut self) -> Res<ast::Formula> {
        let t = self.skip_tok()?;
        Ok(ast::Formula::new_leaf(t.literal()))
    }

    fn parse_number(&mut self, context: &Context) -> Res<ast::Formula> {
        let tok = self.skip_tok()?;
        let n: usize = tok.literal().parse()?;
        if n < context.inner().len() {
            Ok(context.inner()[n].formula().as_ref().clone())
        } else {
            Err(ParseErr::new(
                tok,
                format!("{} >= {} (number of formulas)", n, context.inner().len()),
            ))
        }
    }

    fn parse_unary(&mut self, context: &Context) -> Res<ast::Formula> {
        let operator = self.skip_tok()?.kind();
        let p = operator.precedence();
        Ok(ast::Formula::new_unary(
            operator,
            self.recursive_pratt(p, context)?,
        ))
    }

    fn parse_binary(&mut self, left: ast::Formula, context: &Context) -> Res<ast::Formula> {
        let operator = self.skip_tok()?.kind();
        let p = operator.precedence();
        Ok(ast::Formula::new_binary(
            left,
            operator,
            self.recursive_pratt(p, context)?,
        ))
    }
}
