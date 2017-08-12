use std::collections::{HashMap, BTreeSet};
use std::rc::Rc;
use std::fmt;

//TODO grammar.prod_map should go to Rc<Prod>
//TODO store the terminals into a prop and maintain the same api but with references
use super::{FAKE_GOAL, LAMBDA, EOF, Symbol};


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Production {
    pub from: Symbol,
    pub to: Vec<Symbol>,
}

impl Production {
    pub fn new(from: Symbol, to: Vec<Symbol>) -> Production {
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
    pub goal: Symbol,
    pub productions: Vec<Rc<Production>>,
    prod_map: HashMap<Symbol, Vec<usize>>,
    first_map: HashMap<Symbol, BTreeSet<Symbol>>,
    symbols: Vec<Symbol>,
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
        let mut symbols = vec![goal.clone()];
        for (i, ref prod) in prods.iter().enumerate() {
            assert!(prod.from.is_non_terminal(), "Unexpected terminal in prod.from");
            prod_map.entry(prod.from.clone()).or_insert(vec![]).push(i);
            symbols.push(prod.from.clone());
            //TODO can we improve this?
            for s in &prod.to {
                symbols.push(s.clone());
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

    pub fn get_prods(&self, from: &Symbol) -> Option<Vec<&Rc<Production>>> {
        if from.is_terminal() {
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

    fn calc_first(&self) -> HashMap<Symbol, BTreeSet<Symbol>> {
        use Symbol::*;

        let mut first_map: HashMap<Symbol, BTreeSet<Symbol>> = HashMap::new();
        let mut first_map_snapshot = HashMap::new();

        let lambda_set = vec![T(LAMBDA.to_string())].into_iter().collect();
        let specials = vec![T(EOF.to_string()), T(LAMBDA.to_string())]
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
                                    first_map.get(&prod.to[i - 1]).unwrap().contains(&T(LAMBDA.to_string()))
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

    pub fn first_of(&self, symbols: &Vec<Symbol>) -> Option<BTreeSet<Symbol>> {
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
            if !symbol_first.contains(&Symbol::T(LAMBDA.to_string())) {
                return Some(first);
            }
        }

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
                   vec!["+", "-", LAMBDA, "x"]
                       .into_iter()
                       .map(|s| s.to_string())
                        .map(|s| T(s))
                       .collect::<BTreeSet<Symbol>>())
    }
}
