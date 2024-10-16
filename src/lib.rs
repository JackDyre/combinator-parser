mod parse;

use anyhow::Result;
use nom::combinator::all_consuming;
use parse::{declaration, expression};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombinatorContext(HashSet<Combinator>);

impl CombinatorContext {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    fn validate(c: &Expression, allowed_symbols: &Vec<String>) -> bool {
        c.0.iter().all(|e| match e {
            Element::Item(i) => allowed_symbols.contains(i),
            Element::SubExpression(s) => CombinatorContext::validate(s, allowed_symbols),
            Element::Abstraction(..) => false,
        })
    }

    fn register(&mut self, c: Combinator, allowed_symbols: &Vec<String>) -> Result<Combinator> {
        if !Self::validate(&c.expression, allowed_symbols) {
            anyhow::bail!("Unable to register");
        }
        self.0.insert(c.clone());

        Ok(c)
    }

    pub fn register_strict(&mut self, c: Combinator) -> Result<Combinator> {
        Self::register(
            self,
            c,
            &vec!["S", "K", "I"].iter().map(|s| s.to_string()).collect(),
        )
    }

    pub fn register_loose(&mut self, c: Combinator) -> Result<Combinator> {
        Self::register(
            self,
            c,
            &vec!["S", "K", "I", "B", "C"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }
}

pub trait AbstractionElimination {
    fn contains(&self, variable: &Element) -> bool;
    fn contains_abstraction(&self) -> bool;
    fn abstraction_substitution(&mut self) -> &mut Self;
    fn reduce_parens(&mut self) -> &mut Self;
    fn reduce_expression(&mut self) -> &mut Self;
    fn context_substitution(&mut self, context: &CombinatorContext) -> &mut Self;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Combinator {
    pub name: String,
    pub args: Vec<String>,
    pub expression: Expression,
}

impl Combinator {
    pub fn parse(s: &str) -> Result<Self> {
        let (rest, (name, args)) = declaration(s).unwrap(); // TODO: Propagate
        let (_, exp) = all_consuming(expression)(rest).unwrap();

        let mut c = Self {
            name: name.to_string(),
            args: args.into_iter().map(|a| a.to_string()).collect(),
            expression: Expression(exp),
        };
        c.reduce_parens();

        Ok(c)
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
            while self.contains_abstraction() {
                self.abstraction_substitution();
                self.reduce_parens();
            }
        }

        self
    }

    pub fn apply(&self, name: &str, mut args: Expression) -> Self {
        assert_eq!(self.args.len(), 0);
        let mut c = self.clone();
        c.name = name.to_string();
        c.expression.0.append(&mut args.0);
        c.reduce_expression();
        c
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

    fn reduce_expression(&mut self) -> &mut Self {
        self.expression.reduce_expression();
        self
    }

    fn context_substitution(&mut self, context: &CombinatorContext) -> &mut Self {
        self.expression.context_substitution(context);
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Expression(Vec<Element>);

impl Expression {
    pub fn parse(s: &str) -> Result<Self> {
        let (_, elements) = all_consuming(expression)(s).unwrap();
        Ok(Self(elements))
    }

    pub fn apply(&self, mut args: Expression) -> Self {
        let mut c = self.clone();
        c.0.append(&mut args.0);
        c.reduce_expression();
        c
    }
}

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
        match self.0.first().unwrap() {
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

    fn reduce_expression(&mut self) -> &mut Self {
        loop {
            self.reduce_parens();

            let mut made_substitution = false;

            if self.0.len() > 0 {
                match &self.0[0] {
                    Element::Item(i) if i == "S" && self.0.len() >= 4 => {
                        let x = self.0.remove(1);
                        let y = self.0.remove(1);
                        let z = self.0.remove(1);
                        self.0.remove(0); // Remove S
                        let yz_subexpr = Element::SubExpression(Expression(vec![y, z.clone()]));
                        self.0.insert(0, x);
                        self.0.insert(1, z);
                        self.0.insert(2, yz_subexpr);
                        made_substitution = true;
                    }
                    Element::Item(i) if i == "K" && self.0.len() >= 3 => {
                        let x = self.0.remove(1);
                        self.0.remove(1);
                        self.0.remove(0);
                        self.0.insert(0, x);
                        made_substitution = true;
                    }
                    Element::Item(i) if i == "I" && self.0.len() >= 2 => {
                        let next_elem = self.0.remove(1);
                        self.0.remove(0);
                        self.0.insert(0, next_elem);
                        made_substitution = true;
                    }
                    Element::Item(i) if i == "B" && self.0.len() >= 4 => {
                        let f = self.0.remove(1);
                        let g = self.0.remove(1);
                        let x = self.0.remove(1);
                        self.0.remove(0);
                        let gx_subexpr = Element::SubExpression(Expression(vec![g, x]));
                        self.0.insert(0, f);
                        self.0.insert(1, gx_subexpr);
                        made_substitution = true;
                    }
                    Element::Item(i) if i == "C" && self.0.len() >= 4 => {
                        let f = self.0.remove(1);
                        let x = self.0.remove(1);
                        let y = self.0.remove(1);
                        self.0.remove(0);
                        self.0.insert(0, f);
                        self.0.insert(1, y);
                        self.0.insert(2, x);
                        made_substitution = true;
                    }
                    _ => {}
                }
            }

            if made_substitution {
                self.reduce_parens();
            } else {
                break;
            }
        }
        self.0.iter_mut().for_each(|e| {
            e.reduce_expression();
        });
        self.reduce_parens();
        self
    }

    fn context_substitution(&mut self, context: &CombinatorContext) -> &mut Self {
        context.0.iter().for_each(|d| {
            for element in self.0.iter_mut() {
                match element {
                    Element::Item(i) if *i == d.name => {
                        *element = Element::SubExpression(d.expression.clone());
                    }
                    Element::Item(..) => (),
                    Element::SubExpression(s) => {
                        s.context_substitution(context);
                    }
                    Element::Abstraction(..) => panic!(),
                };
            }
        });
        self.reduce_parens();
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

impl From<Vec<Element>> for Expression {
    fn from(value: Vec<Element>) -> Self {
        Self(value)
    }
}

impl From<Expression> for Vec<Element> {
    fn from(value: Expression) -> Self {
        value.0
    }
}

// impl From<Vec<&str>> for Expression {
//     fn from(value: Vec<&str>) -> Self {
//         value
//             .into_iter()
//             .map(|s| s.into())
//             .collect::<Vec<Element>>()
//             .into()
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
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
                            Self::SubExpression(Expression(vec![
                                Self::SubExpression(Expression(f.to_vec())),
                                g.clone(),
                            ])),
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

    fn reduce_expression(&mut self) -> &mut Self {
        match self {
            Self::Item(_) => (),
            Self::SubExpression(s) => {
                s.reduce_expression();
            }
            Self::Abstraction(_, _) => panic!(),
        };
        self
    }

    fn context_substitution(&mut self, context: &CombinatorContext) -> &mut Self {
        context.0.iter().for_each(|d| match self {
            Self::Item(i) if *i == d.name => {
                *self = Self::SubExpression(d.expression.clone());
            }
            Self::Item(..) => (),
            Self::SubExpression(s) => {
                s.context_substitution(context);
            }
            Self::Abstraction(..) => panic!(),
        });
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

impl From<Vec<Element>> for Element {
    fn from(value: Vec<Element>) -> Self {
        Self::SubExpression(value.into())
    }
}

impl From<&str> for Element {
    fn from(value: &str) -> Self {
        Self::Item(value.to_string())
    }
}

impl From<Vec<&str>> for Element {
    fn from(value: Vec<&str>) -> Self {
        Self::SubExpression(
            value
                .into_iter()
                .map(|s| s.into())
                .collect::<Vec<Element>>()
                .into(),
        )
    }
}

impl From<Expression> for Element {
    fn from(value: Expression) -> Self {
        Self::SubExpression(value)
    }
}
