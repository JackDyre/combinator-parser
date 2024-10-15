use anyhow::{Context, Result};
use combinator_parser::{AbstractionElimination, Combinator, CombinatorContext, Element};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let k: u32 = args
        .get(1)
        .context("Please provide a number as a command-line argument")?
        .parse()?;

    let mut context = CombinatorContext::new();

    let mut succ = Combinator::parse("{Succ}nfx=f(nfx)")?;
    let mut pred = Combinator::parse("{Pred}nfx=n(B(CI)(CIf))(Kx)I")?;
    let mut num = Combinator::parse("0fx=x")?;
    succ.abstraction_elimination();
    pred.abstraction_elimination();
    num.abstraction_elimination();

    for n in 1..=k {
        num = succ.apply(&n.to_string(), vec![num.expression.clone().into()].into());
    }

    let _ = context.register_loose(succ);
    let _ = context.register_loose(pred);

    let display_num = num.apply(
        "InputNum",
        vec!["f", "x"]
            .iter()
            .map(|s| Element::Item(s.to_string()))
            .collect::<Vec<_>>()
            .into(),
    );
    println!("{display_num}");

    let mut ident = Combinator::parse("{Ident}nfx={Pred}({Succ}n)fx")?;
    ident.abstraction_elimination();
    ident.context_substitution(&context);
    println!("{ident}");

    let output = ident.apply("", vec![num.expression.into()].into()).apply(
        "OutputNum",
        vec!["f", "x"]
            .iter()
            .map(|s| Element::Item(s.to_string()))
            .collect::<Vec<_>>()
            .into(),
    );
    println!("{output}");

    Ok(())
}
