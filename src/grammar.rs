use std::collections::{HashMap, BTreeSet};
use std::rc::Rc;

use super::{FAKE_GOAL, Symbol, Production};

#[derive(Debug)]
pub struct Grammar {
    pub goal: Symbol,
    pub productions: Vec<Rc<Production>>,
    prod_map: HashMap<Symbol, Vec<Rc<Production>>>,
    first_map: HashMap<Symbol, BTreeSet<Symbol>>,
    symbols: BTreeSet<Symbol>,
}

impl Grammar {
    pub fn new_simple<T>(goal: T, non_terminals: Vec<T>, prods: Vec<(T, Vec<T>)>) -> Grammar
        where T: Into<String> + Clone
    {
        let non_terminals: BTreeSet<String> = non_terminals.iter().cloned().map(|s| s.into()).collect();

        let prods = prods
            .into_iter()
            .map(|(from, to)| {
                    let from: String = from.into();
                    assert!(non_terminals.contains(&from), "Unexpected terminal in prod.from");
                    let from = Symbol::NT(from);
                    let to = to.into_iter().map(|s| s.into())
                        .map(|s| {
                            if non_terminals.contains(&s) {
                                Symbol::NT(s)
                            } else {
                                Symbol::T(s)
                            }
                        })
                        .collect();

                     Rc::new(Production::new(from, to))
                 })
            .collect();


        let goal: String = goal.into();
        assert!(non_terminals.contains(&goal), "Unexpected terminal goal");
        let goal = Symbol::NT(goal);

        Grammar::new(goal, prods)
    }

    pub fn new(goal: Symbol, prods: Vec<Rc<Production>>) -> Grammar {
        assert!(goal.is_non_terminal(), "Unexpected terminal goal");

        let mut prod_map = HashMap::new();
        let mut symbols = {
            let mut set = BTreeSet::new();
            set.insert(goal.clone());
            set
        };

        for prod in &prods {
            assert!(prod.from.is_non_terminal(), "Unexpected terminal in prod.from");
            prod_map.entry(prod.from.clone()).or_insert(vec![]).push(prod.clone());
            symbols.insert(prod.from.clone());
            for s in &prod.to {
                symbols.insert(s.clone());
            }
        }

        let mut grammar = Grammar {
            goal: goal,
            productions: prods,
            prod_map: prod_map,
            symbols: symbols,
            first_map: HashMap::new(),
        };

        grammar.first_map = grammar.calc_first();
        grammar
    }

    pub fn get_prods(&self, from: &Symbol) -> Option<&Vec<Rc<Production>>> {
        if from.is_terminal() {
            return None;
        }

        self.prod_map.get(from)
    }

    pub fn terminals(&self) -> BTreeSet<Symbol> {
        self.symbols.iter()
            .filter(|s| s.is_terminal())
            .cloned()
            .collect()
    }

    pub fn non_terminals(&self) -> BTreeSet<Symbol> {
        self.symbols.iter()
            .filter(|s| s.is_non_terminal())
            .cloned()
            .collect()
    }

    fn calc_first(&self) -> HashMap<Symbol, BTreeSet<Symbol>> {
        let mut first_map: HashMap<Symbol, BTreeSet<Symbol>> = HashMap::new();
        let mut first_map_snapshot = HashMap::new();

        let lambda_set = vec![Symbol::lambda()].into_iter().collect();
        let specials = vec![Symbol::eof(), Symbol::lambda()]
            .into_iter()
            .collect();

        for t in self.terminals().union(&specials) {
            first_map.insert(t.clone(), vec![t.clone()].into_iter().collect());
        }

        for nt in &self.non_terminals() {
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
                                    first_map.get(&prod.to[i - 1]).unwrap().contains(&Symbol::lambda())
                                })
                    .fold(BTreeSet::new(), |acc, (i, symbol)| {
                        let first_i = first_map.get(symbol).expect("Wrong symbol");
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


    // TODO (potentially) this is a copy paste logic of what happens inside the calc_first
    // can we abstract that?
    pub fn first_of(&self, symbols: &Vec<Symbol>) -> Option<BTreeSet<Symbol>> {
        let lambda_set = vec![Symbol::lambda()].into_iter().collect();

        let first = symbols
            .iter()
            .enumerate()
            .take_while(|&(i, _)| {
                            i == 0 ||
                            self.first_map.get(&symbols[i - 1]).unwrap().contains(&Symbol::lambda())
                        })
            .fold(BTreeSet::new(), |acc, (i, symbol)| {
                let first_i = self.first_map.get(symbol).expect("Wrong symbol");
                let next = if i == symbols.len() - 1 {
                    first_i.iter().cloned().collect()
                } else {
                    first_i.difference(&lambda_set).cloned().collect()
                };

                acc.union(&next).cloned().collect()
            });

        if first.is_empty() { None } else { Some(first) }
    }

    pub fn with_fake_goal(&self) -> Grammar {
        let fake_goal = Symbol::new_nt(FAKE_GOAL);
        let fake_prod = Production::new(fake_goal.clone(), vec![self.goal.clone()]);
        let prods = [vec![Rc::new(fake_prod)], self.productions.clone()].concat();

        Grammar::new(fake_goal, prods)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{LAMBDA};
    fn example_grammar() -> Grammar {
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

        Grammar::new_simple("Goal", non_terminals, prods)
    }

    #[test]
    fn grammar_creation() {
        use Symbol::*;
        let g = example_grammar();

        assert_eq!(g.terminals(),
                   vec!["+", "-", "x", "%", LAMBDA, "(", ")", "num", "name"]
                       .into_iter()
                       .map(|s| s.to_string())
                       .map(|s| T(s))
                       .collect());

        assert_eq!(g.non_terminals(),
                   vec!["Goal", "Expr", "Expr'", "Term", "Term'", "Factor"]
                       .into_iter()
                       .map(|s| s.to_string())
                       .map(|s| NT(s))
                       .collect());

    }

    #[test]
    fn first_of_terminals() {
        let g = example_grammar();

        for t in &g.terminals() {
            assert_eq!(g.first_map.get(t).unwrap(),
                       &vec![t.clone()]
                            .into_iter()
                            .collect());
        }
    }

    #[test]
    fn first_of_non_terminals() {
        use Symbol::*;
        let g = example_grammar();

        let cases = vec![("Goal", vec!["(", "name", "num"]),
                         ("Expr", vec!["(", "name", "num"]),
                         ("Expr'", vec!["+", "-", LAMBDA]),
                         ("Term", vec!["(", "name", "num"]),
                         ("Term'", vec!["x", "%", LAMBDA]),
                         ("Factor", vec!["(", "name", "num"])];

        for &(ref nt, ref first) in &cases {
            let actual = g.first_map.get(&NT(nt.to_string())).unwrap();
            let expected = first
                .into_iter()
                .map(|s| s.to_string())
                .map(|s| T(s))
                .collect::<BTreeSet<Symbol>>();

            assert_eq!(actual,
                       &expected,
                       "\nCase nt {:?}, first {:?}\nActual {:?}\nExpected {:?}",
                       nt,
                       first,
                       actual,
                       expected);
        }
    }

    #[test]
    fn first_of_symbols() {
        use Symbol::*;
        let g = example_grammar();
        assert_eq!(g.first_of(&vec![NT("Expr'".to_string()), T("x".to_string())])
                       .unwrap(),
                   vec!["+", "-", "x"]
                       .into_iter()
                       .map(|s| s.to_string())
                       .map(|s| T(s))
                       .collect::<BTreeSet<Symbol>>())
    }
}
