use crate::token;
use std::{error::Error, fmt};

pub type Res<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct InvalidTokenErr {
    tok: String,
    row: usize,
    col: usize,
}
impl Error for InvalidTokenErr {}
impl InvalidTokenErr {
    pub fn new(tok: String, row: usize, col: usize) -> Box<InvalidTokenErr> {
        Box::new(InvalidTokenErr { tok, col, row })
    }
}
impl fmt::Display for InvalidTokenErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid token [{}:{}]: {}", self.row, self.col, self.tok)
    }
}

#[derive(Debug)]
pub struct ParseErr {
    tok: token::Token,
    message: String,
}
impl Error for ParseErr {}
impl ParseErr {
    pub fn new(tok: token::Token, message: String) -> Box<ParseErr> {
        Box::new(ParseErr { tok, message })
    }
}
impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parse error [{}:{}]: got=`{}` ({:?}): {}",
            self.tok.row(),
            self.tok.col(),
            self.tok.literal(),
            self.tok.kind(),
            self.message
        )
    }
}

#[derive(Debug)]
pub struct IndexOutOfBound {
    message: String,
}
impl Error for IndexOutOfBound {}
impl IndexOutOfBound {
    pub fn new(message: String) -> Box<IndexOutOfBound> {
        Box::new(IndexOutOfBound { message })
    }
}
impl fmt::Display for IndexOutOfBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Index out of bound: {}",
            self.message
        )
    }
}

#[derive(Debug)]
pub struct InternalError {
    message: String,
}
impl Error for InternalError {}
impl InternalError {
    pub fn new(message: String) -> Box<InternalError> {
        Box::new(InternalError { message })
    }
}
impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Interanl error: {}", self.message)
    }
}

#[derive(Debug)]
pub struct InternalErrorTok {
    kind: token::Kind,
    message: String,
}
impl Error for InternalErrorTok {}
impl InternalErrorTok {
    pub fn new(tok: token::Kind, message: String) -> Box<InternalErrorTok> {
        Box::new(InternalErrorTok { kind: tok, message })
    }
}
impl fmt::Display for InternalErrorTok {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Interanl error: got=`{:?}` : {}",
            self.kind, self.message
        )
    }
}
