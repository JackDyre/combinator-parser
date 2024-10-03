use combinator_parser::CombinatorDefinition;
use combinator_parser::CombinatorExpression;

fn main() {
    let c = CombinatorExpression::parse("x".to_string()).unwrap();
    let c = CombinatorDefinition {
        name: "Q".to_string(),
        args: vec!["x".to_string()],
        def: c,
    };
    println!("{c}");
    let c = c.abstraction_elimination();
    println!("{c}");
}
