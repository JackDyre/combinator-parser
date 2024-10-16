use anyhow::Result;
use combinator_parser::{AbstractionElimination, Combinator, CombinatorContext, Expression};

fn prelude(context: &mut CombinatorContext) {
    vec![
        "{True}fg=f",
        "{False}fg=g",
        "{Not}b=b{False}{True}",
        "{Pair}xyb=bxy",
        "{First}p=p{True}",
        "{Second}p=p{False}",
        "{0}=I",
        "{IsZero}={First}",
        "1={Pair}{False}0",
        "{Next}n={Pair}{False}n",
        "{Prev}n={Second}n",
    ]
    .into_iter()
    .for_each(|p| {
        let mut c = Combinator::parse(p).unwrap();
        c.abstraction_elimination().context_substitution(context);
        let _ = context.register_loose(c);
    });
}

fn main() -> Result<()> {
    let mut context = CombinatorContext::new();
    prelude(&mut context);

    let mut c = Expression::parse("{IsZero}1{Yes}{No}")?;
    println!("{c}");
    c.context_substitution(&context);
    println!("{c}");
    c.reduce_expression();
    println!("{c}");

    Ok(())
}
