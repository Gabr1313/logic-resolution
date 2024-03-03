// use crate::token::Token;

// pub struct Unary<'a> {
//     operator: Token<'a>,
//     right: Box<Token<'a>>,
// }
//
// pub struct Binary<'a> {
//     operator: Box<Formula<'a>>,
//     left: Token<'a>,
//     right: Box<Formula<'a>>,
// }
//
// pub struct Leaf<'a> {
//     ident: Token<'a>,
// }
//
// pub enum Formula<'a> {
//     Unary(Unary<'a>),
//     Binary(Binary<'a>),
//     Leaf(Leaf<'a>),
// }
