use std::fmt::{self, Display};
use std::ops::{Deref, Range};
use std::rc::Rc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RcSubstr {
    string: Rc<str>,
    span: Range<usize>,
}

impl RcSubstr {
    pub fn new(string: Rc<str>) -> Self {
        let span = 0..string.len();
        Self { string, span }
    }
    pub fn substr(&self, span: Range<usize>) -> Self {
        // range checked elsewhere by the rust guys
        Self {
            string: Rc::clone(&self.string),
            span: (self.span.start + span.start)..(self.span.start + span.end),
        }
    }
}

impl Deref for RcSubstr {
    type Target = str;
    fn deref(&self) -> &str {
        &self.string[self.span.clone()]
    }
}

impl Display for RcSubstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &*self)
    }
}
