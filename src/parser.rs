//use std::convert::From;
use std::collections::BTreeSet;
use std::fmt;

use super::{Production, Grammar, EOF};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Item {
    pub from: String,
    pub to: Vec<String>,
    pub lookahead: String,
    pub stacktop: usize,
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

    pub fn set_to_string(items: &BTreeSet<Item>) -> String {
        items
            .iter()
            .map(|item| format!("{}", item))
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn set_of_sets_to_string(set: &BTreeSet<BTreeSet<Item>>) -> String {
        set.iter()
            .map(|cc_i| Item::set_to_string(cc_i))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn is_complete(&self) -> bool {
        assert!(self.stacktop <= self.to.len(), "Stacktop out of bounds");
        if self.stacktop == self.to.len() {
            true
        } else {
            false
        }
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
    pub fn closure(&self, items: &BTreeSet<Item>) -> BTreeSet<Item> {
        let mut new_items = items.clone();
        let mut items = BTreeSet::new();

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

            //println!("CLOSURE>>>items     {:?}", Item::set_to_string(&items));
            //println!("CLOSURE>>>new items {:?}\n",
            //Item::set_to_string(&new_items));
        }
        items
    }

    pub fn goto(&self, items: &BTreeSet<Item>, x: String) -> Option<BTreeSet<Item>> {
        let next: BTreeSet<Item> = items
            .iter()
            .filter(|&item| item.stacktop().is_some())
            .filter(|&item| item.stacktop().unwrap() == x)
            .map(|item| item.clone_with_next_stacktop())
            .collect();

        if next.is_empty() {
            return None;
        } else {
            //println!("GOTO>>>next {:?}\n", Item::set_to_string(&next));
            Some(self.closure(&next))
        }
    }

    pub fn build_cc(&self) -> BTreeSet<BTreeSet<Item>> {
        let cc0 = {
            let item = Item::from_production(&self.grammar.productions[0], EOF.to_string());
            let mut set = BTreeSet::new();
            set.insert(item);
            self.closure(&set)
        };

        let mut cc = BTreeSet::new();
        let mut new_cc = {
            let mut set = BTreeSet::new();
            set.insert(cc0);
            set
        };

        while !new_cc.is_empty() {
            cc = cc.union(&new_cc).cloned().collect();
            new_cc.clear();

            //println!("\nBUILD_CC>>>CC \n{}", Item::set_of_sets_to_string(&cc));
            for cc_i in &cc {
                for item in cc_i.iter().filter(|&item| item.stacktop().is_some()) {
                    let stacktop = item.stacktop().unwrap();
                    let next = self.goto(cc_i, stacktop);
                    if next == None {
                        println!("NONE NONE NONE");
                        continue;
                    }
                    let next = next.unwrap();
                    if !cc.contains(&next) {
                        new_cc.insert(next);
                    }
                }
            }
        }

        cc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::EOF;

    fn paretheses_grammar() -> Grammar {
        let non_terminals = vec!["List", "Pair"];

        let prods = vec![("List", vec!["List", "Pair"]),
                         ("List", vec!["Pair"]),

                         ("Pair", vec!["(", "Pair", ")"]),
                         ("Pair", vec!["(", ")"])];

        let g = Grammar::new_simple("List", non_terminals, prods);
        g
    }

    fn example_parser() -> Parser {
        let g = paretheses_grammar();
        Parser { grammar: g.with_fake_goal() }
    }

    #[test]
    fn closure_and_goto_test() {
        let parser = example_parser();
        let first_prod = &parser.grammar.productions[0];
        let item = Item::from_production(first_prod, EOF.to_string());
        let items: BTreeSet<Item> = vec![item].iter().cloned().collect();
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

        let actual = parser.goto(&cc0, "(".to_string()).unwrap();
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

    #[test]
    fn build_cc_test() {
        let parser = example_parser();
        let cc0 = vec![Item::new_simple("FAKE_GOAL", vec!["List"], 0, EOF),
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

        let cc1 = vec![Item::new_simple("FAKE_GOAL", vec!["List"], 1, EOF),

                       Item::new_simple("List", vec!["List", "Pair"], 1, EOF),
                       Item::new_simple("List", vec!["List", "Pair"], 1, "("),

                       Item::new_simple("Pair", vec!["(", "Pair", ")"], 0, EOF),
                       Item::new_simple("Pair", vec!["(", "Pair", ")"], 0, "("),

                       Item::new_simple("Pair", vec!["(", ")"], 0, EOF),
                       Item::new_simple("Pair", vec!["(", ")"], 0, "(")]
                .iter()
                .cloned()
                .collect();

        let cc2 = vec![Item::new_simple("List", vec!["Pair"], 1, EOF),
                       Item::new_simple("List", vec!["Pair"], 1, "(")]
                .iter()
                .cloned()
                .collect();

        let cc3 = vec![Item::new_simple("Pair", vec!["(", "Pair", ")"], 0, ")"),
                       Item::new_simple("Pair", vec!["(", "Pair", ")"], 1, EOF),
                       Item::new_simple("Pair", vec!["(", "Pair", ")"], 1, "("),

                       Item::new_simple("Pair", vec!["(", ")"], 0, ")"),
                       Item::new_simple("Pair", vec!["(", ")"], 1, EOF),
                       Item::new_simple("Pair", vec!["(", ")"], 1, "(")]
                .iter()
                .cloned()
                .collect();

        let cc4 = vec![Item::new_simple("List", vec!["List", "Pair"], 2, EOF),
                       Item::new_simple("List", vec!["List", "Pair"], 2, "(")]
                .iter()
                .cloned()
                .collect();

        let cc5 = vec![Item::new_simple("Pair", vec!["(", "Pair", ")"], 2, EOF),
                       Item::new_simple("Pair", vec!["(", "Pair", ")"], 2, "(")]
                .iter()
                .cloned()
                .collect();

        let cc6 = vec![Item::new_simple("Pair", vec!["(", "Pair", ")"], 0, ")"),
                       Item::new_simple("Pair", vec!["(", "Pair", ")"], 1, ")"),
                       Item::new_simple("Pair", vec!["(", ")"], 0, ")"),
                       Item::new_simple("Pair", vec!["(", ")"], 1, ")")]
                .iter()
                .cloned()
                .collect();

        let cc7 = vec![Item::new_simple("Pair", vec!["(", ")"], 2, EOF),
                       Item::new_simple("Pair", vec!["(", ")"], 2, "(")]
                .iter()
                .cloned()
                .collect();

        let cc8 = vec![Item::new_simple("Pair", vec!["(", "Pair", ")"], 3, EOF),
                       Item::new_simple("Pair", vec!["(", "Pair", ")"], 3, "(")]
                .iter()
                .cloned()
                .collect();

        let cc9 = vec![Item::new_simple("Pair", vec!["(", "Pair", ")"], 2, ")")]
            .iter()
            .cloned()
            .collect();

        let cc10 = vec![Item::new_simple("Pair", vec!["(", ")"], 2, ")")]
            .iter()
            .cloned()
            .collect();

        let cc11 = vec![Item::new_simple("Pair", vec!["(", "Pair", ")"], 3, ")")]
            .iter()
            .cloned()
            .collect();

        let expected_cc: BTreeSet<BTreeSet<Item>> = vec![cc0, cc1, cc2, cc3, cc4, cc5, cc6, cc7,
                                                         cc8, cc9, cc10, cc11]
                .iter()
                .cloned()
                .collect();

        let actual_cc = parser.build_cc();

        assert_eq!(actual_cc.len(),
                   expected_cc.len(),
                   "Should have the same length \nACTUAL   {}\nEXPECTED {}",
                   Item::set_of_sets_to_string(&actual_cc),
                   Item::set_of_sets_to_string(&expected_cc));

        for (actual_items, expected_items) in actual_cc.iter().zip(&expected_cc) {
            assert_eq!(actual_items,
                       expected_items,
                       "\n>>>Actual {}\n>>>Expected {}",
                       Item::set_to_string(actual_items),
                       Item::set_to_string(expected_items));
        }
    }
}
