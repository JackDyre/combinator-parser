mod parse;

use anyhow::Result;
use nom::combinator::all_consuming;
use parse::{declaration, expression};

pub trait AbstractionElimination {
    fn contains(&self, variable: &Element) -> bool;
    fn contains_abstraction(&self) -> bool;
    fn abstraction_substitution(&mut self) -> &mut Self;
    fn reduce_parens(&mut self) -> &mut Self;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Combinator {
    pub name: String,
    pub args: Vec<String>,
    pub expression: Expression,
}

impl Combinator {
    pub fn parse(s: &str) -> Result<Self> {
        let (rest, (name, args)) = declaration(s).unwrap(); // TODO: Propagate
        let (_, exp) = all_consuming(expression)(rest).unwrap();

        Ok(Self {
            name: name.to_string(),
            args: args.into_iter().map(|a| a.to_string()).collect(),
            expression: Expression(exp),
        })
    }

    pub fn add_abstraction(&mut self) -> Option<()> {
        if self.args.len() == 0 {
            return None;
        }
        self.expression = Expression(vec![Element::Abstraction(
            self.expression.clone(),
            self.args.pop()?,
        )]);
        Some(())
    }

    pub fn abstraction_elimination(&mut self) -> &mut Self {
        for _ in 0..self.args.len() {
            self.add_abstraction();
            println!("{self}");
            while self.contains_abstraction() {
                self.abstraction_substitution();
                self.reduce_parens();
                println!("{self}");
            }
        }

        self
    }
}

impl AbstractionElimination for Combinator {
    fn contains(&self, variable: &Element) -> bool {
        self.expression.contains(variable)
    }

    fn contains_abstraction(&self) -> bool {
        self.expression.contains_abstraction()
    }

    fn abstraction_substitution(&mut self) -> &mut Self {
        self.expression.abstraction_substitution();
        self
    }

    fn reduce_parens(&mut self) -> &mut Self {
        self.expression.reduce_parens();
        self
    }
}

impl std::fmt::Display for Combinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.name)?;
        for arg in self.args.iter() {
            write!(f, "{arg} ")?;
        }
        write! {f, "= {}", self.expression}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Expression(Vec<Element>);

impl AbstractionElimination for Expression {
    fn contains(&self, variable: &Element) -> bool {
        self.0.iter().any(|e| e.contains(variable))
    }

    fn contains_abstraction(&self) -> bool {
        self.0.iter().any(|e| e.contains_abstraction())
    }

    fn abstraction_substitution(&mut self) -> &mut Self {
        self.0.iter_mut().for_each(|e| {
            e.abstraction_substitution();
        });
        self
    }

    fn reduce_parens(&mut self) -> &mut Self {
        // Remove parens around the first element if it is a subexpression
        match self.0.get(0).unwrap() {
            Element::Item(_) => {}
            Element::SubExpression(s) => {
                self.0.splice(0..1, s.0.clone());
            }
            Element::Abstraction(_, _) => {}
        };

        // Remove parens around subexpressions with a single element
        self.0.iter_mut().for_each(|e| match e {
            Element::SubExpression(s) => {
                if s.0.len() == 1 {
                    *e = s.0.get(0).unwrap().clone();
                };
            }
            _ => {}
        });

        // Recursively reduce any subexpressions
        self.0.iter_mut().for_each(|e| match e {
            Element::SubExpression(s) => {
                s.reduce_parens();
            }
            Element::Abstraction(s, _) => {
                s.reduce_parens();
            }
            Element::Item(_) => {}
        });
        self
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, element) in self.0.iter().enumerate() {
            if idx != 0 {
                write!(f, " ")?;
            }
            write!(f, "{element}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Element {
    Item(String),
    SubExpression(Expression),
    Abstraction(Expression, String),
}

impl AbstractionElimination for Element {
    fn contains(&self, variable: &Element) -> bool {
        match self {
            Element::Item(i) => Element::Item(i.to_string()) == *variable,
            Element::SubExpression(s) => s.contains(variable),
            Element::Abstraction(s, _) => s.contains(variable),
        }
    }

    fn contains_abstraction(&self) -> bool {
        match self {
            Element::Item(_) => false,
            Element::SubExpression(s) => s.contains_abstraction(),
            Element::Abstraction(_, _) => true,
        }
    }

    fn abstraction_substitution(&mut self) -> &mut Self {
        *self = match self {
            Self::Item(_) => self.clone(),
            Self::SubExpression(s) => {
                s.abstraction_substitution();
                self.clone()
            }
            Self::Abstraction(s, v) => match s.0.as_slice() {
                [f] => match f {
                    Self::Item(i) => match i == v {
                        // [x]_x = I
                        true => Self::Item("I".to_string()),
                        // [f]_x = K f
                        false => Self::SubExpression(Expression(vec![
                            Element::Item("K".to_string()),
                            f.clone(),
                        ])),
                    },
                    _ => panic!(),
                },
                [f @ .., g] => match (
                    f.iter().any(|e| e.contains(&Self::Item(v.to_string()))),
                    g.contains(&Self::Item(v.to_string())),
                    matches!(g, Self::Item(_)),
                ) {
                    (true, true, _) => {
                        // [fg]_x = S [f] [g]
                        Self::SubExpression(Expression(vec![
                            Self::Item("S".to_string()),
                            Self::Abstraction(Expression(f.to_vec()), v.clone()),
                            Self::Abstraction(Expression(vec![g.clone()]), v.clone()),
                        ]))
                    }
                    (true, false, _) => {
                        // [fg]_x = C [f] g
                        Self::SubExpression(Expression(vec![
                            Self::Item("C".to_string()),
                            Self::Abstraction(Expression(f.to_vec()), v.clone()),
                            g.clone(),
                        ]))
                    }
                    (false, true, true) => {
                        // [fx]_x = f
                        Self::SubExpression(Expression(f.to_vec()))
                    }
                    (false, true, false) => {
                        // [fg]_x = B f [g]
                        Self::SubExpression(Expression(vec![
                            Self::Item("B".to_string()),
                            Self::SubExpression(Expression(f.to_vec())),
                            Self::Abstraction(Expression(vec![g.clone()]), v.clone()),
                        ]))
                    }
                    (false, false, _) => {
                        // [f]_x = K f
                        Self::SubExpression(Expression(vec![
                            Self::Item("K".to_string()),
                            Self::SubExpression(Expression(f.to_vec())),
                            g.clone(),
                        ]))
                    }
                },
                _ => panic!(),
            },
        };

        self
    }

    fn reduce_parens(&mut self) -> &mut Self {
        *self = match self {
            Self::Item(_) => self.clone(),
            Self::SubExpression(s) => Self::SubExpression(s.reduce_parens().clone()),
            Self::Abstraction(s, v) => Self::Abstraction(s.reduce_parens().clone(), v.clone()),
        };
        self
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Item(i) => write!(f, "{i}"),
            Self::SubExpression(s) => write!(f, "({s})"),
            Self::Abstraction(s, v) => write!(f, "[{s}]_{v}"),
        }
    }
}
