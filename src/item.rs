use std::collections::BTreeSet;
use std::rc::Rc;
use std::fmt;

//TODO reduce use of clone for non Rc things
//fix to_prod method
//try to return slices from methods that return array of symbols
use super::{FAKE_GOAL, Production};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Item {
    pub prod: Rc<Production>,
    pub lookahead: String,
    pub stacktop: usize,
}

impl Item {
    pub fn new_simple<T>(from: T, to: Vec<T>, stacktop: usize, lookahead: T) -> Item
        where T: Into<String> + Clone
    {
        let to = to.into_iter().map(|s| s.into()).collect();
        let prod = Production::new(from.into(), to);
        Item::new(Rc::new(prod), stacktop, lookahead.into())
    }

    pub fn new(prod: Rc<Production>, stacktop: usize, lookahead: String) -> Item {
        Item {
            prod: prod,
            stacktop: stacktop,
            lookahead: lookahead,
        }
    }

    pub fn from_production(prod: Rc<Production>, lookahead: String) -> Item {
        Item::new(prod, 0, lookahead)
    }

    //TODO rename, probably not needed anymore
    pub fn to_prod(&self) -> Rc<Production> {
        self.prod.clone()
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
        assert!(self.stacktop <= self.prod.to.len(), "Stacktop out of bounds");
        if self.stacktop == self.prod.to.len() {
            true
        } else {
            false
        }
    }

    pub fn is_terminator(&self) -> bool {
        self.prod.from.as_str() == FAKE_GOAL && self.stacktop == 1 && self.is_complete()
    }

    pub fn stacktop(&self) -> Option<String> {
        if self.stacktop == self.prod.to.len() {
            // Item complete
            return None;
        } else if self.stacktop < self.prod.to.len() {
            return self.prod.to.get(self.stacktop).map(|s| s.clone());
        } else {
            panic!("Stacktop out of bounds")
        }
    }

    //TODO return a slice
    pub fn after_stacktop(&self) -> Vec<String> {
        self.prod.to[self.stacktop + 1..].to_vec()
    }

    pub fn after_stacktop_and_lookahead(&self) -> Vec<String> {
        let mut rest = self.after_stacktop();
        rest.push(self.lookahead.clone());
        rest
    }

    pub fn clone_with_next_stacktop(&self) -> Item {
        let mut item = self.clone();
        if item.stacktop() == None {
            panic!("Attempting to push item's stacktop when the item is already complete {:?}",
                   self);
        }

        item.stacktop += 1;
        item
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_str: String = if self.stacktop() == None {
            format!("{} •", self.prod.to.join(" "))
        } else {
            self.prod.to
                .iter()
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
