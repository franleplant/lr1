use std::collections::{HashMap, BTreeSet};
use super::{Grammar, Production, EOF, Item};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    Accept,
    Reduce(Production),
    Shift(BTreeSet<Item>),
}


//TODO we need to avoid copying in goto and action and Action so much
//Probably better to identify cc's as an index, and then refrence through out like that
//because is expensive to copy it
//I think that the only way right now is by using indices
//so
//action: HashMap<(usize, usize), Vec<Action>>
//where the first usize is the index corresponding to cc_i
//and the second is the index corresponding to a symbol in the grammar
//
//the same happens to goto
//
//so we basically need
//## Grammar
//- Symbol <-> number
//- Production <-> number
//
//## Parser
//- cc_i <-> number
//
//We can easily get number -> Any with Array
//we can easily get Any -> number with a HashMap
//
//What about if we combie these two in a single data structure: CanonicalSet?
//The contract is:
//- cc.get_cc(i) -> cc_i
//- cc.get_i(cc_i) -> i
//- cc.insert(cc)
//- cc.iter (access to the array)
//
//
//what about using Rc?
//because the above is very complicated
//
//
//
//TODO
//print tables in a human readable way
#[derive(Debug)]
pub struct Parser {
    grammar: Grammar,
    cc: BTreeSet<BTreeSet<Item>>,
    goto: HashMap<(BTreeSet<Item>, String), BTreeSet<BTreeSet<Item>>>,
    action: HashMap<(BTreeSet<Item>, String), BTreeSet<Action>>,
}

impl Parser {
    pub fn new(g: Grammar) -> Parser {
        Parser {
            grammar: g,
            cc: BTreeSet::new(),
            goto: HashMap::new(),
            action: HashMap::new(),
        }
    }

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

    pub fn goto(&self, items: &BTreeSet<Item>, x: &String) -> Option<BTreeSet<Item>> {
        let next: BTreeSet<Item> = items
            .iter()
            .filter(|&item| item.stacktop().is_some())
            .filter(|&item| &item.stacktop().unwrap() == x)
            .map(|item| item.clone_with_next_stacktop())
            .collect();

        if next.is_empty() {
            return None;
        } else {
            //println!("GOTO>>>next {:?}\n", Item::set_to_string(&next));
            Some(self.closure(&next))
        }
    }

    pub fn build_cc(&mut self) {
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
                    let next = self.goto(cc_i, &stacktop);
                    if next == None {
                        println!("NONE NONE NONE");
                        continue;
                    }
                    let next = next.unwrap();
                    if !cc.contains(&next) {
                        new_cc.insert(next.clone());
                    }

                    if self.grammar.is_terminal(&stacktop) {
                        let entry = self.action
                            .entry((cc_i.clone(), stacktop))
                            .or_insert(BTreeSet::new());

                        entry.insert(Action::Shift(next));

                    } else {
                        let entry = self.goto.entry((cc_i.clone(), stacktop)).or_insert(BTreeSet::new());
                        entry.insert(next);
                    }

                }
            }
        }

        self.cc = cc;
    }

    //TODO merge this with build_cc
    pub fn build_action(&mut self) {
        for cc_i in &self.cc {
            for item in cc_i.iter().filter(|&item| item.is_complete()) {
                let entry = self.action
                    .entry((cc_i.clone(), item.lookahead.clone()))
                    .or_insert(BTreeSet::new());

                if item.is_terminator() {
                    entry.insert(Action::Accept);
                } else {
                    entry.insert(Action::Reduce(item.to_prod()));
                }
            }
        }
    }

    pub fn print_tables(&self) {
        let mut index_to_cc_i = vec![];
        let mut cc_i_to_index: HashMap<BTreeSet<Item>, usize> = HashMap::new();
        for (i, cc_i) in self.cc.iter().enumerate() {
            index_to_cc_i.push(cc_i.clone());
            cc_i_to_index.insert(cc_i.clone(), i);
            println!("{:<4} {}",i, Item::set_to_string(cc_i));
        }
        println!("\n");


        println!("ACTION");
        println!("======");
        let mut rows: Vec<Vec<String>> = vec![];

        let mut terminals = self.grammar.terminals
            .iter()
            .cloned()
            .collect();

        let mut first_row = vec!["".to_string(), EOF.to_string()];
        first_row.append(&mut terminals);

        rows.push(first_row);


        let mut terminals = vec![EOF.to_string()];
        terminals.append(&mut self.grammar.terminals.iter().cloned().collect());

        for (i, cc_i) in self.cc.iter().enumerate() {
            let mut row = vec![i.to_string()];
            for t in &terminals {
                let action = self.action.get(&(cc_i.clone(), t.clone()));
                if action == None {
                    row.push("".to_string());
                } else {
                    let s = action.unwrap()
                        .iter()
                        .map(|a| {
                            match a {
                                &Action::Accept => "Accept".to_string(),
                                &Action::Reduce(ref prod) => format!("{}", prod),
                                &Action::Shift(ref cc_i) => format!("Shift({})", cc_i_to_index.get(cc_i).unwrap()),
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(", ");

                    row.push(s);
                }
            }

            rows.push(row);
        }


        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i == 0 {
                    print!("{:<4}", cell);
                } else {
                    print!("{:<30}", cell);
                }
            }
            println!("");
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{FAKE_GOAL, EOF};

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
        Parser::new(g.with_fake_goal())
    }

    #[test]
    fn closure_and_goto_test() {
        let parser = example_parser();
        let first_prod = &parser.grammar.productions[0];
        let item = Item::from_production(first_prod, EOF.to_string());
        let items: BTreeSet<Item> = vec![item].iter().cloned().collect();
        let cc0 = parser.closure(&items);

        let actual = &cc0;
        let expected = vec![Item::new_simple(FAKE_GOAL, vec!["List"], 0, EOF),
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

        let actual = parser.goto(&cc0, &"(".to_string()).unwrap();
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
        let mut parser = example_parser();
        let cc0 = vec![Item::new_simple(FAKE_GOAL, vec!["List"], 0, EOF),
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

        let cc1 = vec![Item::new_simple(FAKE_GOAL, vec!["List"], 1, EOF),

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

        parser.build_cc();
        let actual_cc = parser.cc.clone();

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

    #[test]
    fn tables_test() {
        let mut parser = example_parser();
        parser.build_cc();
        parser.build_action();

        parser.print_tables();
        //println!("{:?}", parser.goto);
        //println!("{:?}", parser.action);

    }
}
