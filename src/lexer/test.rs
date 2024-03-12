use std::rc::Rc;

use super::Lexer;
use crate::error::{InvalidTokenErr, Res};
use crate::token;

#[test]
fn test_lexer() {
    let buffer = "
x => y
x| y;
x & y
x <=>y   ;
!
~x ;
x&y
(x | y) & z!
is_al_num <=> Is_Al_NuM
-12
??
09
<<=> y
x <y
^
";
    let expected: &[Res<token::Token>] = &[
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("x".to_string()),
            2,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::Implies,
            Rc::new("=>".to_string()),
            2,
            3,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("y".to_string()),
            2,
            6,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("x".to_string()),
            3,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::Or,
            Rc::new("|".to_string()),
            3,
            2,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("y".to_string()),
            3,
            4,
        )),
        Ok(token::Token::new(
            token::Kind::SemiColon,
            Rc::new(";".to_string()),
            3,
            5,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("x".to_string()),
            4,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::And,
            Rc::new("&".to_string()),
            4,
            3,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("y".to_string()),
            4,
            5,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("x".to_string()),
            5,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::Equiv,
            Rc::new("<=>".to_string()),
            5,
            3,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("y".to_string()),
            5,
            6,
        )),
        Ok(token::Token::new(
            token::Kind::SemiColon,
            Rc::new(";".to_string()),
            5,
            10,
        )),
        Ok(token::Token::new(
            token::Kind::Bang,
            Rc::new("!".to_string()),
            6,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::Not,
            Rc::new("~".to_string()),
            7,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("x".to_string()),
            7,
            2,
        )),
        Ok(token::Token::new(
            token::Kind::SemiColon,
            Rc::new(";".to_string()),
            7,
            4,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("x".to_string()),
            8,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::And,
            Rc::new("&".to_string()),
            8,
            2,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("y".to_string()),
            8,
            3,
        )),
        Ok(token::Token::new(
            token::Kind::ParenL,
            Rc::new("(".to_string()),
            9,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("x".to_string()),
            9,
            2,
        )),
        Ok(token::Token::new(
            token::Kind::Or,
            Rc::new("|".to_string()),
            9,
            4,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("y".to_string()),
            9,
            6,
        )),
        Ok(token::Token::new(
            token::Kind::ParenR,
            Rc::new(")".to_string()),
            9,
            7,
        )),
        Ok(token::Token::new(
            token::Kind::And,
            Rc::new("&".to_string()),
            9,
            9,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("z".to_string()),
            9,
            11,
        )),
        Ok(token::Token::new(
            token::Kind::Bang,
            Rc::new("!".to_string()),
            9,
            12,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("is_al_num".to_string()),
            10,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::Equiv,
            Rc::new("<=>".to_string()),
            10,
            11,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("Is_Al_NuM".to_string()),
            10,
            15,
        )),
        Ok(token::Token::new(
            token::Kind::Minus,
            Rc::new("-".to_string()),
            11,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::Number,
            Rc::new("12".to_string()),
            11,
            2,
        )),
        Ok(token::Token::new(
            token::Kind::Question,
            Rc::new("?".to_string()),
            12,
            1,
        )),
        Ok(token::Token::new(
            token::Kind::Question,
            Rc::new("?".to_string()),
            12,
            2,
        )),
        Ok(token::Token::new(
            token::Kind::Number,
            Rc::new("09".to_string()),
            13,
            1,
        )),
        Err(InvalidTokenErr::new("<<=>".to_string(), 14, 1)),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("y".to_string()),
            14,
            6,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("x".to_string()),
            15,
            1,
        )),
        Err(InvalidTokenErr::new("<".to_string(), 15, 3)),
        Ok(token::Token::new(
            token::Kind::Identifier,
            Rc::new("y".to_string()),
            15,
            4,
        )),
        Err(InvalidTokenErr::new("^".to_string(), 16, 1)),
        Ok(token::Token::new(
            token::Kind::Eof,
            Rc::new("".to_string()),
            17,
            1,
        )),
    ];
    let mut lex = Lexer::new();
    lex.load_bytes(buffer.to_string());

    for exp in expected {
        let e = format!("{:?}", exp);
        let l = format!("{:?}", lex.next_tok());
        if e != l {
            panic!("exptected=`{e}`, got=`{l}`")
        }
    }
}
