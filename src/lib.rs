mod parse;

use anyhow::Result;
use nom::combinator::all_consuming;
use parse::{declaration, expression};

pub trait AbstractionElimination {
    fn contains(&self, variable: &Element) -> bool;
    fn contains_abstraction(&self) -> bool;
    fn abstraction_substitution(&mut self) -> &mut Self;
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
        let _ = self.0.iter_mut().map(|e| e.abstraction_substitution());
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
        match self {
            Self::Item(_) => self,
            Self::SubExpression(s) => {
                s.abstraction_substitution();
                self
            }
            Self::Abstraction(s, v) => match s.0.as_slice() {
                [f] => match f {
                    Self::Item(i) => match i == v {
                        true => todo!(),  // [x]_x = I
                        false => todo!(), // [f]_x = K f
                    },
                    _ => panic!(),
                },
                [f @ .., g] => match (
                    f.iter().any(|e| e.contains(&Self::Item(v.to_string()))),
                    g.contains(&Self::Item(v.to_string())),
                    matches!(g, Self::Item(_)),
                ) {
                    (true, true, _) => todo!(),      // [fg]_x = S [f] [g]
                    (true, false, _) => todo!(),     // [fg]_x = C [f] g
                    (false, true, true) => todo!(),  // [fx]_x = f
                    (false, true, false) => todo!(), // [fg]_x = B f [g]
                    (false, false, _) => todo!(),    // [f]_x = K f
                },
                _ => panic!(),
            },
        }
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
