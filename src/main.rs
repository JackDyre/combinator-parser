use combinator_parser::CombinatorExpression;

fn main() {
    let c = CombinatorExpression::parse("S(KS)K".to_string()).unwrap();
    dbg!(c);
}
