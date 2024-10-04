use std::fmt;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CombinatorExpression {
    pub exp: Vec<Combinator>,
}

impl CombinatorExpression {
    pub fn parse(string: String) -> anyhow::Result<Self> {
        let mut exp: Vec<Combinator> = Vec::new();
        let mut sub_exp: Vec<char> = Vec::new();
        let mut paren_depth: i32 = 0;

        for char in string.chars() {
            if char == '(' {
                paren_depth += 1;
                if paren_depth > 1 {
                    sub_exp.push(char)
                }
                continue;
            }
            if char == ')' {
                paren_depth -= 1;
                if paren_depth == 0 {
                    let sub_string: String = sub_exp.clone().into_iter().collect();
                    exp.push(Combinator::Expression(Self::parse(sub_string).unwrap()));
                    sub_exp = Vec::new();
                } else {
                    sub_exp.push(char)
                }
                continue;
            }
            if paren_depth == 0 {
                exp.push(Combinator::Single(char.to_string()))
            }
            if paren_depth > 0 {
                sub_exp.push(char);
            }
        }

        Ok(Self { exp })
    }

    pub fn contains(&self, var: &str) -> bool {
        self.exp.iter().any(|a| match a {
            Combinator::Single(e) => e == var,
            Combinator::Expression(e) => e.contains(var),
            Combinator::Abstraction {
                exp: e,
                variable: _,
            } => e.contains(var),
        })
    }

    pub fn contains_abstraction(&self) -> bool {
        self.exp.iter().any(|a| match a {
            Combinator::Single(_) => false,
            Combinator::Expression(e) => e.contains_abstraction(),
            Combinator::Abstraction {
                exp: _,
                variable: _,
            } => true,
        })
    }

    pub fn abstraction_elimination(mut self) -> Self {
        for elem in self.exp.iter_mut() {
            *elem = match elem {
                Combinator::Single(_) => continue,
                Combinator::Expression(e) => {
                    Combinator::Expression(e.clone().abstraction_elimination())
                }
                Combinator::Abstraction {
                    exp: e,
                    variable: v,
                } => match e.exp.as_slice() {
                    [f] => match f {
                        Combinator::Single(fucking_variable_name) => {
                            if fucking_variable_name == v {
                                Combinator::Single("I".to_string())
                            } else {
                                // K f maybe
                                Combinator::Expression(CombinatorExpression {
                                    exp: vec![Combinator::Single("K".to_string()), f.clone()],
                                })
                            }
                        }
                        Combinator::Expression(other_fucking_variable_name) => {
                            if other_fucking_variable_name.contains(&v.clone()) {
                                Combinator::Abstraction {
                                    exp: other_fucking_variable_name
                                        .clone()
                                        .abstraction_elimination(),
                                    variable: v.clone(),
                                }
                            } else {
                                Combinator::Expression(CombinatorExpression {
                                    exp: vec![
                                        Combinator::Single("K".to_string()),
                                        Combinator::Expression(other_fucking_variable_name.clone()),
                                    ],
                                })
                            }
                        }
                        Combinator::Abstraction {
                            exp: _,
                            variable: _,
                        } => panic!(),
                    },
                    [f @ .., g] => {
                        if f.iter().any(|q| q.contains(v.clone())) && g.contains(v.clone()) {
                            Combinator::Expression(CombinatorExpression {
                                exp: vec![
                                    Combinator::Single("S".to_string()),
                                    Combinator::Abstraction {
                                        exp: CombinatorExpression { exp: f.to_vec() },
                                        variable: v.clone(),
                                    },
                                    Combinator::Abstraction {
                                        exp: CombinatorExpression {
                                            exp: vec![g.clone()],
                                        },
                                        variable: v.clone(),
                                    },
                                ],
                            })
                        } else if !f.iter().any(|q| q.contains(v.clone())) && g.contains(v.clone())
                        {
                            if let Combinator::Single(_) = g {
                                Combinator::Expression(CombinatorExpression { exp: f.to_vec() })
                            } else {
                                Combinator::Expression(CombinatorExpression {
                                    exp: vec![
                                        Combinator::Single("B".to_string()),
                                        Combinator::Expression(CombinatorExpression {
                                            exp: f.to_vec(),
                                        }),
                                        Combinator::Abstraction {
                                            exp: CombinatorExpression {
                                                exp: vec![g.clone()],
                                            },
                                            variable: v.clone(),
                                        },
                                    ],
                                })
                            }
                        } else if f.iter().any(|q| q.contains(v.clone())) && !g.contains(v.clone())
                        {
                            // C [f] g
                            Combinator::Expression(CombinatorExpression {
                                exp: vec![
                                    Combinator::Single("C".to_string()),
                                    Combinator::Abstraction {
                                        exp: CombinatorExpression { exp: f.to_vec() },
                                        variable: v.clone(),
                                    },
                                    g.clone(),
                                ],
                            })
                        } else {
                            // K (fg)
                            Combinator::Expression(CombinatorExpression {
                                exp: vec![
                                    Combinator::Single("K".to_string()),
                                    Combinator::Expression(CombinatorExpression {
                                        exp: {
                                            let mut tv = f.to_vec();
                                            tv.push(g.clone());
                                            tv
                                        },
                                    }),
                                ],
                            })
                        }
                    }
                    _ => continue,
                },
            }
        }

        self
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Combinator {
    Single(String),
    Expression(CombinatorExpression),
    Abstraction {
        exp: CombinatorExpression,
        variable: String,
    },
}

impl Combinator {
    pub fn contains(&self, var: String) -> bool {
        match self {
            Self::Single(e) => *e == var,
            Self::Expression(e) => e.contains(&var),
            Self::Abstraction {
                exp: e,
                variable: _,
            } => e.contains(&var),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CombinatorDefinition {
    pub def: CombinatorExpression,
    pub args: Vec<String>,
    pub name: String,
}

impl CombinatorDefinition {
    pub fn new(args: Vec<String>, exp: String, name: String) -> anyhow::Result<Self> {
        let def = CombinatorExpression::parse(exp)?;
        Ok(Self { def, args, name })
    }

    /// $Fabc...wx:=f$ is equivalent to $Fabc...w:=[f]_x$ where
    /// - $[x]_x=I$
    /// - $[f]_x=K$, if $f$ does not conain $x$
    /// - $[gx]_x=g$, if $g$ does not conatin $x$
    /// - $[fg]_x=Bf[g]_x$, if $f$ does not contain $x$ but $g$ does
    /// - $[fg]_x=C[f]_xg$, if $f$ does contains $x$ but $g$ does not
    /// - $[fg]_x=S[f]_x[g]_x$, if both $f$ and $g$ contain $x$
    pub fn abstraction_elimination(mut self) -> Self {
        if self.args.len() == 0 {
            return self;
        }
        let arg = self.args.pop().unwrap();

        self.def = vec![Combinator::Abstraction {
            exp: self.def,
            variable: arg.clone(),
        }]
        .into();

        while self.def.contains_abstraction() {
            self.def = self.clone().def.abstraction_elimination()
        }

        self
    }
}

impl fmt::Display for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Combinator::Single(s) => write!(f, "{}", s),
            Combinator::Expression(exp) => write!(f, "({})", exp),
            Combinator::Abstraction { exp, variable } => write!(f, "[{}]_{}", exp, variable),
        }
    }
}

impl fmt::Display for CombinatorExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for combinator in self.exp.iter() {
            write!(f, "{}", combinator)?;
        }
        Ok(())
    }
}

impl From<Vec<Combinator>> for CombinatorExpression {
    fn from(value: Vec<Combinator>) -> Self {
        Self { exp: value }
    }
}

impl fmt::Display for CombinatorDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        for arg in self.args.iter() {
            write!(f, "{}", arg)?;
        }
        write!(f, "={}", self.def)?;

        Ok(())
    }
}
