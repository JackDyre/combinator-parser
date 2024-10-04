use combinator_parser::CombinatorDefinition;
use combinator_parser::CombinatorExpression;

fn main() {
    let expression = "x".to_string();
    let c = CombinatorExpression::parse(expression).unwrap();
    let c = CombinatorDefinition {
        name: "Q".to_string(),
        args: vec!["x".to_string()],
        def: c,
    };
    println!("{c}");
    let c = c.abstraction_elimination();
    println!("{c}");
}
