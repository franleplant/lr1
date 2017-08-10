use std::collections::{HashMap, BTreeSet};
use std::rc::Rc;
use super::{Grammar, Production, EOF, Item};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    Accept,
    Reduce(Production),
    Shift(Rc<BTreeSet<Item>>),
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
    cc: BTreeSet<Rc<BTreeSet<Item>>>,
    goto_map: HashMap<(Rc<BTreeSet<Item>>, String), BTreeSet<Rc<BTreeSet<Item>>>>,
    action: HashMap<(Rc<BTreeSet<Item>>, String), BTreeSet<Action>>,

    index_to_cc: Vec<Rc<BTreeSet<Item>>>,
    cc_to_index: HashMap<Rc<BTreeSet<Item>>, usize>,
}

impl Parser {
    pub fn new(g: Grammar) -> Parser {
        Parser {
            grammar: g,
            cc: BTreeSet::new(),
            goto_map: HashMap::new(),
            action: HashMap::new(),

            index_to_cc: Vec::new(),
            cc_to_index: HashMap::new(),
        }
    }

    pub fn insert_cc(&mut self, cc_i: Rc<BTreeSet<Item>>) {
        assert!(!self.cc_to_index.contains_key(&cc_i));
        let index = self.index_to_cc.len();

        self.index_to_cc.push(cc_i.clone());
        self.cc_to_index.insert(cc_i.clone(), index);
    }

    pub fn closure(&self, items: &BTreeSet<Item>) -> Rc<BTreeSet<Item>> {
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

        Rc::new(items)
    }

    pub fn goto(&self, items: &BTreeSet<Item>, x: &String) -> Option<Rc<BTreeSet<Item>>> {
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
            set.insert(cc0.clone());
            set
        };

        self.insert_cc(cc0);

        while !new_cc.is_empty() {
            cc = cc.union(&new_cc).cloned().collect();
            new_cc.clear();

            // TODO we have a method and a attribute goto, disambiugate this
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
                        let is_new = new_cc.insert(next.clone());
                        if is_new {
                            self.insert_cc(next.clone());
                        }
                    }

                    if self.grammar.is_terminal(&stacktop) {
                        let entry = self.action
                            .entry((cc_i.clone(), stacktop))
                            .or_insert(BTreeSet::new());

                        entry.insert(Action::Shift(next.clone()));

                    } else {
                        let entry = self.goto_map
                            .entry((cc_i.clone(), stacktop))
                            .or_insert(BTreeSet::new());
                        entry.insert(next.clone());
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

    // TODO what about cc_vec instead of index_to_cc
    // what about cc_map instead of cc_to_index
    pub fn print_tables(&self) {
        println!("CC");
        println!("======");

        for (i, cc_i) in self.index_to_cc.iter().enumerate() {
            println!("{:<4} {}", i, Item::set_to_string(cc_i));
        }
        println!("\n");


        println!("ACTION");
        println!("======");
        let mut rows: Vec<Vec<String>> = vec![];

        let mut first_row = vec!["".to_string(), EOF.to_string()];
        first_row.append(&mut self.grammar.terminals.iter().cloned().collect());

        rows.push(first_row);


        let mut terminals = vec![EOF.to_string()];
        terminals.append(&mut self.grammar.terminals.iter().cloned().collect());

        for (i, cc_i) in self.index_to_cc.iter().enumerate() {
            let mut row = vec![i.to_string()];
            for t in &terminals {
                let action = self.action.get(&(cc_i.clone(), t.clone()));
                if action == None {
                    row.push("".to_string());
                } else {
                    let s = action
                        .unwrap()
                        .iter()
                        .map(|a| match a {
                                 &Action::Accept => "Accept".to_string(),
                                 &Action::Reduce(ref prod) => format!("{}", prod),
                                 &Action::Shift(ref cc_i) => format!("Shift({})", self.cc_to_index.get(cc_i).unwrap()),
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

        println!("\n");
        println!("GOTO");
        println!("====");
        let mut rows: Vec<Vec<String>> = vec![];

        let mut first_row = vec!["".to_string()];
        first_row.append(&mut self.grammar.non_terminals.iter().cloned().collect());

        rows.push(first_row);

        for (i, cc_i) in self.cc.iter().enumerate() {
            let mut row = vec![i.to_string()];
            for nt in &self.grammar.non_terminals {
                let next = self.goto_map.get(&(cc_i.clone(), nt.clone()));
                if next == None {
                    row.push("".to_string());
                } else {
                    row.push(self.cc_to_index.get(cc_i).unwrap().to_string());
                }
            }
            rows.push(row);
        }

        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i == 0 {
                    print!("{:<4}", cell);
                } else {
                    print!("{:<10}", cell);
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

        let expected = Rc::new(expected);

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
        let expected = Rc::new(expected);

        assert_eq!(actual,
                   expected,
                   "\n\n>>>actual {}\n>>>expected {}",
                   Item::set_to_string(&actual),
                   Item::set_to_string(&expected));
    }

    #[test]
    fn goto_test2() {
        let parser = example_parser();
        let cc_vec = paretheses_cc();

        let col = vec!["Goal", "List", "Pair", "(", ")", EOF];

        let expected = vec![
            [None, Some(cc_vec[1].clone()), Some(cc_vec[2].clone()), Some(cc_vec[3].clone()), None, None],
            [None, None, Some(cc_vec[4].clone()), Some(cc_vec[3].clone()), None, None],

            [None, None, None, None, None, None],
            [None, None, Some(cc_vec[5].clone()), Some(cc_vec[6].clone()), Some(cc_vec[7].clone()), None],

            [None, None, None, None, None, None],
            [None, None, None, None, Some(cc_vec[8].clone()), None],

            [None, None, Some(cc_vec[9].clone()), Some(cc_vec[6].clone()), Some(cc_vec[10].clone()), None],
            [None, None, None, None, None, None],

            [None, None, None, None, None, None],
            [None, None, None, None, Some(cc_vec[11].clone()), None],

            [None, None, None, None, None, None],
            [None, None, None, None, None, None],
        ];

        for (i, row) in expected.iter().enumerate() {
            for (j, e) in row.iter().enumerate() {
                let a = parser.goto(&cc_vec[i], &col[j].to_string());

                assert_eq!(a.clone(), e.clone(), "\nFrom {:?}\nActual {:?}\nExpected {:?}",
                           Item::set_to_string(&cc_vec[i]),
                           a.clone().map(|a| Item::set_to_string(&a)),
                           e.clone().map(|e| Item::set_to_string(&e)),
                        );
            }
        }
    }

    #[test]
    fn build_cc_test() {
        let mut parser = example_parser();

        let expected_cc: BTreeSet<Rc<BTreeSet<Item>>> = paretheses_cc()
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

    fn paretheses_cc() -> Vec<Rc<BTreeSet<Item>>> {
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

        vec![cc0, cc1, cc2, cc3, cc4, cc5, cc6, cc7, cc8, cc9, cc10, cc11]
            .iter()
            .cloned()
            .map(|s| Rc::new(s))
            .collect()
    }
}
