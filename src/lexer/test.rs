use super::Lexer;
use crate::error::{InvalidTokenErr, Res};
use crate::token;

#[test]
fn test_lexer() {
    let buffer = "\
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
exit
help
";
    let expected: &[Res<token::Token>] = &[
        Ok(token::Token::new(token::Kind::Identifier, "x".into(), 1, 1)),
        Ok(token::Token::new(token::Kind::Implies, "=>".into(), 1, 3)),
        Ok(token::Token::new(token::Kind::Identifier, "y".into(), 1, 6)),
        Ok(token::Token::new(token::Kind::Separator, "\n".into(), 1, 7)),
        Ok(token::Token::new(token::Kind::Identifier, "x".into(), 2, 1)),
        Ok(token::Token::new(token::Kind::Or, "|".into(), 2, 2)),
        Ok(token::Token::new(token::Kind::Identifier, "y".into(), 2, 4)),
        Ok(token::Token::new(token::Kind::Separator, ";".into(), 2, 5)),
        Ok(token::Token::new(token::Kind::Separator, "\n".into(), 2, 6)),
        Ok(token::Token::new(token::Kind::Identifier, "x".into(), 3, 1)),
        Ok(token::Token::new(token::Kind::And, "&".into(), 3, 3)),
        Ok(token::Token::new(token::Kind::Identifier, "y".into(), 3, 5)),
        Ok(token::Token::new(token::Kind::Separator, "\n".into(), 3, 6)),
        Ok(token::Token::new(token::Kind::Identifier, "x".into(), 4, 1)),
        Ok(token::Token::new(token::Kind::Equiv, "<=>".into(), 4, 3)),
        Ok(token::Token::new(token::Kind::Identifier, "y".into(), 4, 6)),
        Ok(token::Token::new(token::Kind::Separator, ";".into(), 4, 10)),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            4,
            11,
        )),
        Ok(token::Token::new(token::Kind::Bang, "!".into(), 5, 1)),
        Ok(token::Token::new(token::Kind::Separator, "\n".into(), 5, 2)),
        Ok(token::Token::new(token::Kind::Not, "~".into(), 6, 1)),
        Ok(token::Token::new(token::Kind::Identifier, "x".into(), 6, 2)),
        Ok(token::Token::new(token::Kind::Separator, ";".into(), 6, 4)),
        Ok(token::Token::new(token::Kind::Separator, "\n".into(), 6, 5)),
        Ok(token::Token::new(token::Kind::Identifier, "x".into(), 7, 1)),
        Ok(token::Token::new(token::Kind::And, "&".into(), 7, 2)),
        Ok(token::Token::new(token::Kind::Identifier, "y".into(), 7, 3)),
        Ok(token::Token::new(token::Kind::Separator, "\n".into(), 7, 4)),
        Ok(token::Token::new(token::Kind::ParenL, "(".into(), 8, 1)),
        Ok(token::Token::new(token::Kind::Identifier, "x".into(), 8, 2)),
        Ok(token::Token::new(token::Kind::Or, "|".into(), 8, 4)),
        Ok(token::Token::new(token::Kind::Identifier, "y".into(), 8, 6)),
        Ok(token::Token::new(token::Kind::ParenR, ")".into(), 8, 7)),
        Ok(token::Token::new(token::Kind::And, "&".into(), 8, 9)),
        Ok(token::Token::new(
            token::Kind::Identifier,
            "z".into(),
            8,
            11,
        )),
        Ok(token::Token::new(token::Kind::Bang, "!".into(), 8, 12)),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            8,
            13,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            "is_al_num".into(),
            9,
            1,
        )),
        Ok(token::Token::new(token::Kind::Equiv, "<=>".into(), 9, 11)),
        Ok(token::Token::new(
            token::Kind::Identifier,
            "Is_Al_NuM".into(),
            9,
            15,
        )),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            9,
            24,
        )),
        Ok(token::Token::new(token::Kind::Minus, "-".into(), 10, 1)),
        Ok(token::Token::new(token::Kind::Number, "12".into(), 10, 2)),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            10,
            4,
        )),
        Ok(token::Token::new(token::Kind::Question, "?".into(), 11, 1)),
        Ok(token::Token::new(token::Kind::Question, "?".into(), 11, 2)),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            11,
            3,
        )),
        Ok(token::Token::new(token::Kind::Number, "09".into(), 12, 1)),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            12,
            3,
        )),
        Err(InvalidTokenErr::new("<<=>".to_string(), 13, 1)),
        Ok(token::Token::new(
            token::Kind::Identifier,
            "y".into(),
            13,
            6,
        )),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            13,
            7,
        )),
        Ok(token::Token::new(
            token::Kind::Identifier,
            "x".into(),
            14,
            1,
        )),
        Err(InvalidTokenErr::new("<y".to_string(), 14, 3)),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            14,
            5,
        )),
        Err(InvalidTokenErr::new("^".to_string(), 15, 1)),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            15,
            2,
        )),
        Ok(token::Token::new(token::Kind::Exit, "exit".into(), 16, 1)),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            16,
            5,
        )),
        Ok(token::Token::new(token::Kind::Help, "help".into(), 17, 1)),
        Ok(token::Token::new(
            token::Kind::Separator,
            "\n".into(),
            17,
            5,
        )),
        Ok(token::Token::new(token::Kind::Eoi, "".into(), 18, 1)),
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
