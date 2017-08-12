use std::fmt;

use super::{Symbol};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Production {
    pub from: Symbol,
    pub to: Vec<Symbol>,
}

impl Production {
    pub fn new(from: Symbol, to: Vec<Symbol>) -> Production {
        Production { from: from, to: to }
    }
}

impl fmt::Display for Production {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{} -> {}",
               self.from,
               self.to
                   .iter()
                   .map(|s| format!("{:?}", s))
                   .collect::<Vec<String>>()
                   .join(" "))
    }
}
