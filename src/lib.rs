pub mod parse;

use anyhow::Result;
use nom::combinator::all_consuming;
use parse::{declaration, expression};

pub trait VariableContainer {
    fn contains(&self, variable: &Element) -> bool;
    fn contains_abstraction(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Combinator {
    pub name: String,
    pub args: Vec<String>,
    pub expression: Expression,
}

impl Combinator {
    pub fn parse(s: &str) -> Result<Self> {
        let (rest, (name, args)) = declaration(s).unwrap();
        let (_, exp) = all_consuming(expression)(rest).unwrap();

        Ok(Self {
            name: name.to_string(),
            args: args.into_iter().map(|a| a.to_string()).collect(),
            expression: Expression(exp),
        })
    }
}

impl VariableContainer for Combinator {
    fn contains(&self, variable: &Element) -> bool {
        self.expression.contains(variable)
    }

    fn contains_abstraction(&self) -> bool {
        self.expression.contains_abstraction()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Expression(Vec<Element>);

impl VariableContainer for Expression {
    fn contains(&self, variable: &Element) -> bool {
        self.0.iter().any(|e| e.contains(variable))
    }

    fn contains_abstraction(&self) -> bool {
        self.0.iter().any(|e| e.contains_abstraction())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Element {
    Item(String),
    SubExpression(Vec<Element>),
    Abstraction(Vec<Element>, String),
}

impl VariableContainer for Element {
    fn contains(&self, variable: &Element) -> bool {
        match self {
            Element::Item(i) => Element::Item(i.to_string()) == *variable,
            Element::SubExpression(s) => s.iter().any(|e| e.contains(variable)),
            Element::Abstraction(s, _) => s.iter().any(|e| e.contains(variable)),
        }
    }

    fn contains_abstraction(&self) -> bool {
        match self {
            Element::Item(_) => false,
            Element::SubExpression(s) => s.iter().any(|e| e.contains_abstraction()),
            Element::Abstraction(_, _) => true,
        }
    }
}
