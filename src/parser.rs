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

    pub fn stacktop(&self) -> &String {
        self.to.get(self.stacktop).expect("Stacktop out of bounds")
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

            for item in &items {
                let prods = self.grammar.get_prods(item.stacktop());
                if prods == None {
                    continue;
                }

                for prod in prods.unwrap() {
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
}
