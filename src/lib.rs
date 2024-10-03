use std::fmt;

#[derive(Debug, Eq, PartialEq)]
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
}

#[derive(Debug, Eq, PartialEq)]
pub enum Combinator {
    Single(String),
    Expression(CombinatorExpression),
    Abstraction {
        exp: CombinatorExpression,
        variable: String,
    },
}
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

        while self.def.exp.iter().any(|a| match a {
            Combinator::Abstraction {
                exp: _,
                variable: _,
            } => true,
            _ => false,
        }) {
            break;
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
