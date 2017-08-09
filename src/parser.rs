//use std::convert::From;
use std::collections::HashSet;

use super::{Production, Grammar};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub from: String,
    pub to: Vec<String>,
    pub stacktop: usize,
    pub lookahead: String,
}

impl Item {
    pub fn new(from: String, to: Vec<String>, stacktop: usize, lookahead: String) -> Item {
        Item {
            from: from,
            to: to,
            stacktop: stacktop,
            lookahead: lookahead,
        }
    }

    pub fn from_production(prod: &Production, lookahead: String) -> Item {
        let &Production { ref from, ref to } = prod;
        Item::new(from.clone(), to.clone(), 0, lookahead)
    }

    pub fn stacktop(&self) -> Option<String> {
        if self.stacktop == self.to.len() {
            // Item complete
            return None;
        } else if self.stacktop < self.to.len() {
            return self.to.get(self.stacktop).map(|s| s.clone());
        } else {
            panic!("Stacktop out of bounds")
        }
    }

    //TODO return a slice
    pub fn after_stacktop(&self) -> Vec<String> {
        self.to[self.stacktop + 1..].to_vec()
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

pub struct Parser {
    grammar: Grammar,
}

impl Parser {
    pub fn closure(&self, items: HashSet<Item>) -> HashSet<Item> {
        let mut new_items = items;
        let mut items = HashSet::new();

        while !new_items.is_empty() {
            items = items.union(&new_items).cloned().collect();
            new_items.clear();

            let filtered_items =
                items
                    .iter()
                    .filter(|item| item.stacktop().is_some())
                    .filter(|item| self.grammar.is_terminal(&item.stacktop().unwrap()))
                    .filter(|item| self.grammar.get_prods(&item.stacktop().unwrap()).is_some());


            for item in filtered_items {
                for prod in self.grammar.get_prods(&item.stacktop().unwrap()).unwrap() {
                    let first = self.grammar.first_of(&item.after_stacktop_and_lookahead());
                    if first == None {
                        continue;
                    }

                    for b in first.unwrap() {
                        let item = Item::from_production(prod, b.clone());
                        new_items.insert(item);
                    }
                }
            }
        }
        items
    }

    pub fn goto(&self, items: HashSet<Item>, x: String) -> HashSet<Item> {
        let next: HashSet<Item> = items
            .iter()
            .filter(|&item| item.stacktop().is_some())
            .filter(|&item| item.stacktop().unwrap() == x)
            .map(|item| item.clone_with_next_stacktop())
            .collect();

        self.closure(next)
    }
}
