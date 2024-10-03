#[derive(Debug)]
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
                continue;
            }
            if char == ')' {
                paren_depth -= 1;
                if paren_depth == 0 {
                    let sub_string: String = sub_exp.clone().into_iter().collect();
                    exp.push(Combinator::Expression(Self::parse(sub_string).unwrap()))
                }
                sub_exp = Vec::new();
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
}

#[derive(Debug)]
pub enum Combinator {
    Single(String),
    Expression(CombinatorExpression),
    Abstraction {
        exp: CombinatorExpression,
        variable: String,
    },
}

