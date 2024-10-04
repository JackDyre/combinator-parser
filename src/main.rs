use combinator_parser::CombinatorDefinition;
use combinator_parser::CombinatorExpression;

fn main() {
    let expression = "f(x)".to_string();
    let c = CombinatorExpression::parse(expression).unwrap();
    let c = CombinatorDefinition {
        name: "Q".to_string(),
        args: "fgx"
            .chars()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .into_iter()
            .map(|d| d.to_string())
            .collect(),
        def: c,
    };

    let mut thingy = c;

    println!("{thingy}");
    for _ in 1..=thingy.args.len() {
        thingy = thingy.clone().abstraction_elimination();
        println!("{thingy}");
    }
}
