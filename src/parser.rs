//use std::convert::From;
use std::collections::HashSet;
use std::fmt;

use super::{Production, Grammar};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub from: String,
    pub to: Vec<String>,
    pub stacktop: usize,
    pub lookahead: String,
}

impl Item {
    pub fn new_simple<T: Into<String> + Clone>(from: T,
                                               to: Vec<T>,
                                               stacktop: usize,
                                               lookahead: T)
                                               -> Item {
        let to = to.iter().cloned().map(|s| s.into()).collect();
        Item::new(from.into(), to, stacktop, lookahead.into())
    }

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

    pub fn set_to_string(items: &HashSet<Item>) -> String {
        items
            .iter()
            .map(|item| format!("{}", item))
            .collect::<Vec<String>>()
            .join(" ")
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

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_str: String = if self.stacktop() == None {
            format!("{} •", self.to.join(" "))
        } else {
            self.to
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

        write!(f, "[{} -> {}, {}]", self.from, to_str, self.lookahead)
    }
}


pub struct Parser {
    grammar: Grammar,
}

impl Parser {
    // TODO probably this functions can be standalone
    // since they only need access to the grammar
    pub fn closure(&self, items: &HashSet<Item>) -> HashSet<Item> {
        let mut new_items = items.clone();
        let mut items = HashSet::new();

        while !new_items.is_subset(&items) {
            items = items.union(&new_items).cloned().collect();
            new_items.clear();

            let filtered_items =
                items
                    .iter()
                    .filter(|item| item.stacktop().is_some())
                    .filter(|item| self.grammar.is_non_terminal(&item.stacktop().unwrap()))
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

            println!("CLOSURE>>>items {:?}", Item::set_to_string(&items));
            println!("CLOSURE>>>new items {:?}\n", Item::set_to_string(&items));
        }
        items
    }

    pub fn goto(&self, items: &HashSet<Item>, x: String) -> HashSet<Item> {
        let next: HashSet<Item> = items
            .iter()
            .filter(|&item| item.stacktop().is_some())
            .filter(|&item| item.stacktop().unwrap() == x)
            .map(|item| item.clone_with_next_stacktop())
            .collect();

        println!("GOTO>>>next {:?}\n", Item::set_to_string(&next));
        self.closure(&next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::EOF;

    fn example_parser() -> Parser {

        let non_terminals = vec!["List", "Pair"];

        let prods = vec![("List", vec!["List", "Pair"]),
                         ("List", vec!["Pair"]),

                         ("Pair", vec!["(", "Pair", ")"]),
                         ("Pair", vec!["(", ")"])];

        let g = Grammar::new_simple("List", non_terminals, prods);

        Parser { grammar: g.with_fake_goal() }
    }

    #[test]
    fn closure_and_goto_test() {
        let parser = example_parser();
        let first_prod = &parser.grammar.productions[0];
        let item = Item::from_production(first_prod, EOF.to_string());
        let items: HashSet<Item> = vec![item].iter().cloned().collect();
        let cc0 = parser.closure(&items);

        let actual = &cc0;
        let expected = vec![Item::new_simple("FAKE_GOAL", vec!["List"], 0, EOF),
                            Item::new_simple("List", vec!["List", "Pair"], 0, EOF),
                            Item::new_simple("List", vec!["List", "Pair"], 0, "("),
                            Item::new_simple("List", vec!["Pair"], 0, EOF),
                            Item::new_simple("List", vec!["Pair"], 0, "("),

                            Item::new_simple("Pair", vec!["(", "Pair", ")"], 0, EOF),
                            Item::new_simple("Pair", vec!["(", "Pair", ")"], 0, "("),
                            Item::new_simple("Pair", vec!["(", ")"], 0, EOF),
                            Item::new_simple("Pair", vec!["(", ")"], 0, "(")]
                .iter()
                .cloned()
                .collect();

        assert_eq!(actual,
                   &expected,
                   "\n\n>>>actual {}\n>>>expected {}",
                   Item::set_to_string(&actual),
                   Item::set_to_string(&expected));

        let actual = parser.goto(&cc0, "(".to_string());
        let expected = vec![Item::new_simple("Pair", vec!["(", "Pair", ")"], 1, EOF),
                            Item::new_simple("Pair", vec!["(", "Pair", ")"], 1, "("),

                            Item::new_simple("Pair", vec!["(", ")"], 1, EOF),
                            Item::new_simple("Pair", vec!["(", ")"], 1, "("),

                            Item::new_simple("Pair", vec!["(", "Pair", ")"], 0, ")"),
                            Item::new_simple("Pair", vec!["(", ")"], 0, ")")]
                .iter()
                .cloned()
                .collect();

        assert_eq!(actual,
                   expected,
                   "\n\n>>>actual {}\n>>>expected {}",
                   Item::set_to_string(&actual),
                   Item::set_to_string(&expected));
    }
}
