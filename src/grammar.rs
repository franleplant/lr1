use std::collections::{HashMap, BTreeSet};
use std::rc::Rc;
use std::fmt;

//TODO figure out a way of encoding being a terminal or not in the symbol itself
//perhaps by an enum, and by clasifying that when creating the grammar
//
//TODO make calc_first better
use super::{LAMBDA, EOF};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Production {
    pub from: String,
    pub to: Vec<String>,
}

impl Production {
    pub fn new(from: String, to: Vec<String>) -> Production {
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

#[derive(Debug)]
pub struct Grammar {
    pub goal: String,
    pub productions: Vec<Rc<Production>>,
    pub non_terminals: BTreeSet<String>,
    pub terminals: BTreeSet<String>,
    prod_map: HashMap<String, Vec<usize>>,
    first_map: HashMap<String, BTreeSet<String>>,
}

impl Grammar {
    pub fn new_simple<T>(goal: T, non_terminals: Vec<T>, prods: Vec<(T, Vec<T>)>) -> Grammar
        where T: Into<String> + Clone
    {
        let non_terminals = non_terminals.iter().cloned().map(|s| s.into()).collect();
        let prods = prods
            .iter()
            .cloned()
            .map(|(from, to)| {
                     let to = to.iter().cloned().map(|s| s.into()).collect();
                     Rc::new(Production::new(from.into(), to))
                 })
            .collect();

        Grammar::new(goal.into(), non_terminals, prods)
    }

    //TODO let prod map be a map to Rc<Prod>
    pub fn new(goal: String, non_terminals: Vec<String>, prods: Vec<Rc<Production>>) -> Grammar {
        let mut prod_map = HashMap::new();
        for (i, ref prod) in prods.iter().enumerate() {
            prod_map.entry(prod.from.clone()).or_insert(vec![]).push(i);
        }

        let mut non_terminals: BTreeSet<String> = non_terminals.iter().cloned().collect();
        non_terminals.insert(goal.clone());

        let mut grammar = Grammar {
            goal: goal,
            non_terminals: non_terminals,
            terminals: BTreeSet::new(),
            productions: prods,
            prod_map: prod_map,
            first_map: HashMap::new(),
        };

        grammar.terminals = grammar.calc_terminals();
        grammar.first_map = grammar.calc_first();

        grammar
    }

    pub fn get_prods(&self, from: &String) -> Option<Vec<&Rc<Production>>> {
        if self.is_terminal(from) {
            return None;
        }

        self.prod_map
            .get(from)
            .map(|prod_indices| {
                     prod_indices
                         .iter()
                         .map(|prod_index| {
                                  self.productions.get(*prod_index).expect("Bad prod index!")
                              })
                         .collect()
                 })

    }

    fn calc_terminals(&self) -> BTreeSet<String> {
        self.productions
            .iter()
            .flat_map(|prod| {
                          let mut symbols = prod.to.clone();
                          symbols.push(prod.from.clone());
                          symbols
                      })
            .filter(|symbol| !self.non_terminals.contains(symbol))
            .collect()
    }

    fn calc_first(&self) -> HashMap<String, BTreeSet<String>> {

        let mut first_map: HashMap<String, BTreeSet<String>> = HashMap::new();
        let mut first_map_snapshot = HashMap::new();

        let lambda_set = vec![LAMBDA.to_string()].into_iter().collect();
        let specials = vec![EOF.to_string(), LAMBDA.to_string()]
            .into_iter()
            .collect();

        for t in self.terminals.union(&specials) {
            first_map.insert(t.clone(), vec![t.clone()].into_iter().collect());
        }

        for nt in &self.non_terminals {
            first_map.insert(nt.clone(), vec![].into_iter().collect());
        }

        while first_map != first_map_snapshot {
            first_map_snapshot = first_map.clone();
            for prod in &self.productions {

                let rhs = prod.to
                    .iter()
                    .enumerate()
                    .take_while(|&(i, _)| {
                                    i == 0 ||
                                    first_map.get(&prod.to[i - 1]).unwrap().contains(LAMBDA)
                                })
                    .fold(BTreeSet::new(), |acc, (i, symbol)| {
                        let first_i = first_map.get(symbol).unwrap();
                        let next = if i == prod.to.len() - 1 {
                            first_i.iter().cloned().collect()
                        } else {
                            first_i.difference(&lambda_set).cloned().collect()
                        };

                        acc.union(&next).cloned().collect()
                    });

                if let Some(first) = first_map.get_mut(&prod.from) {
                    *first = first.union(&rhs).cloned().collect();
                }
            }
        }

        first_map
    }

    pub fn is_non_terminal(&self, symbol: &String) -> bool {
        self.non_terminals.contains(symbol)
    }

    pub fn is_terminal(&self, symbol: &String) -> bool {
        self.terminals.contains(symbol)
    }

    pub fn first_of(&self, symbols: &Vec<String>) -> Option<BTreeSet<String>> {
        let mut first = BTreeSet::new();

        let first_by_symbol = symbols
            .iter()
            .map(|symbol| self.first_map.get(symbol))
            .map(|opt| {
                     opt.expect(&*format!("Something went wrong when finding frist of {:?}",
                                         symbols))
                 });

        for symbol_first in first_by_symbol {
            first = first.union(symbol_first).cloned().collect();
            if !symbol_first.contains(LAMBDA) {
                return Some(first);
            }
        }

        if first.is_empty() { None } else { Some(first) }
    }

    pub fn with_fake_goal(&self) -> Grammar {
        let fake_goal = "FAKE_GOAL".to_string();
        let non_terminals = {
            let mut non_terminals = self.non_terminals.clone();
            non_terminals.insert(fake_goal.clone());
            non_terminals.iter().cloned().collect()
        };

        let fake_prod = Production::new(fake_goal.clone(), vec![self.goal.clone()]);
        let prods = [vec![Rc::new(fake_prod)], self.productions.clone()].concat();

        Grammar::new(fake_goal, non_terminals, prods)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_creation() {
        let non_terminals = vec!["Goal", "Expr", "Expr'", "Term", "Term'", "Factor"];

        let prods = vec![("Goal", vec!["Expr"]),

                         ("Expr", vec!["Term", "Expr'"]),

                         ("Expr'", vec!["+", "Term", "Expr'"]),
                         ("Expr'", vec!["-", "Term", "Expr'"]),
                         ("Expr'", vec![LAMBDA]),

                         ("Term", vec!["Factor", "Term'"]),

                         ("Term'", vec!["x", "Factor", "Term'"]),
                         ("Term'", vec!["%", "Factor", "Term'"]),
                         ("Term'", vec![LAMBDA]),

                         ("Factor", vec!["(", "Expr", ")"]),
                         ("Factor", vec!["num"]),
                         ("Factor", vec!["name"])];

        let g = Grammar::new_simple("Goal", non_terminals, prods);

        assert_eq!(g.terminals,
                   vec!["+", "-", "x", "%", LAMBDA, "(", ")", "num", "name"]
                       .iter()
                       .map(|s| s.to_string())
                       .collect());

        assert_eq!(g.non_terminals,
                   vec!["Goal", "Expr", "Expr'", "Term", "Term'", "Factor"]
                       .iter()
                       .map(|s| s.to_string())
                       .collect());

        for t in &g.terminals {
            assert_eq!(g.first_map.get(t).unwrap(),
                       &vec![t.clone()]
                            .iter()
                            .cloned()
                            .collect::<BTreeSet<String>>());
        }


        let cases = vec![("Goal", vec!["(", "name", "num"]),
                         ("Expr", vec!["(", "name", "num"]),
                         ("Expr'", vec!["+", "-", LAMBDA]),
                         ("Term", vec!["(", "name", "num"]),
                         ("Term'", vec!["x", "%", LAMBDA]),
                         ("Factor", vec!["(", "name", "num"])];

        for &(ref nt, ref first) in &cases {
            let actual = g.first_map.get(&nt.to_string()).unwrap();
            let expected = first
                .iter()
                .cloned()
                .map(|s| s.to_string())
                .collect::<BTreeSet<String>>();

            assert_eq!(actual,
                       &expected,
                       "\nCase nt {:?}, first {:?}\nActual {:?}\nExpected {:?}",
                       nt,
                       first,
                       actual,
                       expected);
        }

        assert_eq!(g.first_of(&vec!["Expr'".to_string(), "x".to_string()])
                       .unwrap(),
                   vec!["+", "-", LAMBDA, "x"]
                       .iter()
                       .cloned()
                       .map(|s| s.to_string())
                       .collect::<BTreeSet<String>>())
    }
}
