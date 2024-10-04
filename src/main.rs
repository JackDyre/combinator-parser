use combinator_parser::CombinatorDefinition;
use combinator_parser::CombinatorExpression;

fn main() {
    let expression = "f(yx)".to_string();
    let c = CombinatorExpression::parse(expression).unwrap();
    let c = CombinatorDefinition {
        name: "T".to_string(),
        args: vec!["x"].into_iter().map(|d| d.to_string()).collect(),
        def: c,
    };
    println!("{c}");
    let c = c.abstraction_elimination();
    println!("{c}");
    let c = c.abstraction_elimination();
    println!("{c}");
    let c = c.abstraction_elimination();
    println!("{c}");
}
