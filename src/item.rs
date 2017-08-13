use std::collections::BTreeSet;
use std::rc::Rc;
use std::fmt;

use super::{FAKE_GOAL, Production, Symbol, Grammar};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Item {
    pub prod: Rc<Production>,
    pub lookahead: Symbol,
    pub stacktop: usize,
}

impl Item {
    pub fn from_str<T>(from: T, to: Vec<T>, stacktop: usize, lookahead: T, g: &Grammar) -> Item
    where
        T: Into<String> + Clone,
    {
        let non_terminals: BTreeSet<String> = g.non_terminals()
            .iter()
            .map(|s| s.to_string())
            .cloned()
            .collect();

        let from: String = from.into();
        assert!(
            non_terminals.contains(&from),
            "From needs to be a Non Terminal! got {} and NonTerminals {:?}",
            from,
            non_terminals
        );

        let from = Symbol::NT(from);
        let to = to.into_iter()
            .map(|s| s.into())
            .map(|s| if non_terminals.contains(&s) {
                Symbol::NT(s)
            } else {
                Symbol::T(s)
            })
            .collect();

        let lookahead: String = lookahead.into();
        let lookahead = if non_terminals.contains(&lookahead) {
            Symbol::NT(lookahead)
        } else {
            Symbol::T(lookahead)
        };

        let prod = Production::new(from, to);
        Item::new(Rc::new(prod), stacktop, lookahead)
    }

    pub fn new(prod: Rc<Production>, stacktop: usize, lookahead: Symbol) -> Item {
        Item {
            prod: prod,
            stacktop: stacktop,
            lookahead: lookahead,
        }
    }

    pub fn from_production(prod: Rc<Production>, lookahead: Symbol) -> Item {
        Item::new(prod, 0, lookahead)
    }

    pub fn set_to_string(items: &BTreeSet<Item>) -> String {
        items
            .iter()
            .map(|item| format!("{}", item))
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn set_of_sets_to_string(set: &BTreeSet<Rc<BTreeSet<Item>>>) -> String {
        set.iter()
            .map(|cc_i| Item::set_to_string(cc_i))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn is_complete(&self) -> bool {
        assert!(
            self.stacktop <= self.prod.to.len(),
            "Stacktop out of bounds"
        );
        if self.stacktop == self.prod.to.len() {
            true
        } else {
            false
        }
    }

    pub fn is_terminator(&self) -> bool {
        self.prod.from.as_str() == FAKE_GOAL && self.stacktop == 1 && self.is_complete()
    }

    pub fn stacktop(&self) -> Option<&Symbol> {
        if self.stacktop == self.prod.to.len() {
            // Item complete
            return None;
        } else if self.stacktop < self.prod.to.len() {
            return self.prod.to.get(self.stacktop);
        } else {
            panic!("Stacktop out of bounds")
        }
    }

    pub fn after_stacktop(&self) -> &[Symbol] {
        &self.prod.to[self.stacktop + 1..]
    }

    pub fn after_stacktop_and_lookahead(&self) -> Vec<Symbol> {
        let head = self.after_stacktop();
        let tail = &[self.lookahead.clone()];
        head.iter().chain(tail.iter()).cloned().collect()
    }

    pub fn clone_with_next_stacktop(&self) -> Item {
        let mut item = self.clone();
        if item.is_complete() {
            panic!(
                "Attempting to push item's stacktop when the item is already complete {:?}",
                self
            );
        }

        item.stacktop += 1;
        item
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to: Vec<String> = self.prod
            .to
            .iter()
            .map(|s| s.to_string())
            .cloned()
            .collect();
        let to_str: String = if self.stacktop() == None {
            format!("{} •", to.join(" "))
        } else {
            to.iter()
                .enumerate()
                .map(|(i, s)| if i == self.stacktop {
                    format!("• {}", s)
                } else {
                    s.clone()
                })
                .collect::<Vec<String>>()
                .join(" ")
        };

        write!(f, "[{} -> {}, {}]", self.prod.from, to_str, self.lookahead)
    }
}
