use std::collections::{HashMap, HashSet};

use super::{LAMBDA, EOF};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Production {
    pub from: String,
    pub to: Vec<String>,
}

impl Production {
    pub fn new(from: String, to: Vec<String>) -> Production {
        Production { from: from, to: to }
    }
}

pub struct Grammar {
    pub goal: String,
    pub productions: Vec<Production>,
    pub non_terminals: HashSet<String>,
    pub terminals: HashSet<String>,
    prod_map: HashMap<String, Vec<usize>>,
    first_map: HashMap<String, HashSet<String>>,
}

impl Grammar {
    pub fn new_simple<T: Into<String> + Clone>(goal: T,
                                               non_terminals: Vec<T>,
                                               prods: Vec<(T, Vec<T>)>)
                                               -> Grammar {
        let non_terminals = non_terminals.iter().cloned().map(|s| s.into()).collect();
        let prods = prods
            .iter()
            .cloned()
            .map(|(from, to)| {
                     let to = to.iter().cloned().map(|s| s.into()).collect();
                     Production::new(from.into(), to)
                 })
            .collect();

        Grammar::new(goal.into(), non_terminals, prods)
    }

    pub fn new(goal: String, non_terminals: Vec<String>, prods: Vec<Production>) -> Grammar {
        let mut prod_map = HashMap::new();
        for (i, &Production { ref from, .. }) in prods.iter().enumerate() {
            prod_map.entry(from.clone()).or_insert(vec![]).push(i);
        }

        let mut non_terminals: HashSet<String> = non_terminals.iter().cloned().collect();
        non_terminals.insert(goal.clone());

        let mut grammar = Grammar {
            goal: goal,
            non_terminals: non_terminals,
            terminals: HashSet::new(),
            productions: prods,
            prod_map: prod_map,
            first_map: HashMap::new(),
        };

        grammar.terminals = grammar.calc_terminals();
        grammar.first_map = grammar.calc_first();

        grammar
    }

    pub fn get_prods(&self, from: &String) -> Option<Vec<&Production>> {
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

    fn calc_terminals(&self) -> HashSet<String> {
        self.productions
            .iter()
            .cloned()
            .flat_map(|Production { from, to }| {
                          let mut symbols = to;
                          symbols.push(from);
                          symbols
                      })
            .filter(|symbol| !self.non_terminals.contains(symbol))
            .collect()
    }

    fn calc_first(&self) -> HashMap<String, HashSet<String>> {
        let mut first_map: HashMap<String, HashSet<String>> = HashMap::new();

        let specials: HashSet<String> = vec![EOF.to_string(), LAMBDA.to_string()]
            .iter()
            .cloned()
            .collect();

        for t in self.terminals.union(&specials) {
            first_map.insert(t.clone(), vec![t.clone()].iter().cloned().collect());
        }

        for nt in &self.non_terminals {
            first_map.insert(nt.clone(), vec![].iter().cloned().collect());
        }

        let lambda_set: HashSet<String> = vec![LAMBDA.to_string()].iter().cloned().collect();

        let mut first_map_snapshot = HashMap::new();
        while first_map != first_map_snapshot {
            first_map_snapshot = first_map.clone();
            for &Production { ref from, ref to } in &self.productions {
                //TODO try to make it more rusty
                let mut rhs: HashSet<String> = first_map
                    .get(&to[0])
                    .unwrap()
                    .difference(&lambda_set)
                    .cloned()
                    .collect();

                let mut i = 0;
                while first_map.get(&to[i]).unwrap().contains(LAMBDA) && i < to.len() - 1 {
                    let next: HashSet<String> = first_map
                        .get(&to[i + 1])
                        .unwrap()
                        .difference(&lambda_set)
                        .cloned()
                        .collect();
                    rhs = rhs.union(&next).cloned().collect();
                    i += 1;
                }

                if i == to.len() - 1 && first_map.get(&to[to.len() - 1]).unwrap().contains(LAMBDA) {
                    rhs = rhs.union(&lambda_set).cloned().collect();
                }

                if let Some(first) = first_map.get_mut(from) {
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

    pub fn first_of(&self, symbols: &Vec<String>) -> Option<HashSet<String>> {
        let mut first = HashSet::new();

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
                       &vec![t.clone()].iter().cloned().collect::<HashSet<String>>());
        }


        let cases = vec![("Goal", vec!["(", "name", "num"]),
                         ("Expr", vec!["(", "name", "num"]),
                         ("Expr'", vec!["+", "-", LAMBDA]),
                         ("Term", vec!["(", "name", "num"]),
                         ("Term'", vec!["x", "%", LAMBDA]),
                         ("Factor", vec!["(", "name", "num"])];

        for &(ref nt, ref first) in &cases {
            assert_eq!(g.first_map.get(&nt.to_string()).unwrap(),
                       &first
                            .iter()
                            .cloned()
                            .map(|s| s.to_string())
                            .collect::<HashSet<String>>(),
                       "Case nt {:?}, first {:?}",
                       nt,
                       first);
        }

        assert_eq!(g.first_of(&vec!["Expr'".to_string(), "x".to_string()])
                       .unwrap(),
                   vec!["+", "-", LAMBDA, "x"]
                       .iter()
                       .cloned()
                       .map(|s| s.to_string())
                       .collect::<HashSet<String>>())
    }
}
