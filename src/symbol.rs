use std::fmt;
use super::{EOF, LAMBDA};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    T(String),
    NT(String),
}

impl Symbol {
    pub fn new_t(s: &str) -> Symbol {
        Symbol::T(s.to_string())
    }

    pub fn new_nt(s: &str) -> Symbol {
        Symbol::NT(s.to_string())
    }

    pub fn eof() -> Symbol {
        Symbol::T(EOF.to_string())
    }

    pub fn lambda() -> Symbol {
        Symbol::T(LAMBDA.to_string())
    }

    pub fn is_terminal(&self) -> bool {
        match self {
            &Symbol::T(_) => true,
            _ => false,
        }
    }

    pub fn is_non_terminal(&self) -> bool {
        match self {
            &Symbol::NT(_) => true,
            _ => false,
        }
    }

    pub fn to_string(&self) -> &String {
        match self {
            &Symbol::T(ref s) => s,
            &Symbol::NT(ref s) => s,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            &Symbol::T(ref s) => s,
            &Symbol::NT(ref s) => s,
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
