use anyhow::{Context, Result};
use combinator_parser::{Combinator, Element};
use std::env;

fn main() -> Result<()> {
    let mut succ = Combinator::parse("{Succ}nfx=f(nfx)")?;
    succ.abstraction_elimination();
    println!("{succ}");

    let mut num = Combinator::parse("0fx=x")?;
    num.abstraction_elimination();

    // Read k from command line arguments
    let args: Vec<String> = env::args().collect();
    let k: u32 = args
        .get(1)
        .context("Please provide a number as a command-line argument")?
        .parse()?;

    for n in 1..=k {
        num = succ.apply(&n.to_string(), vec![num.expression.clone().into()].into());
    }
    println!("{num}");

    // let f_num = num.apply(
    //     "FinalNum",
    //     vec!["f", "x"]
    //         .iter()
    //         .map(|s| Element::Item(s.to_string()))
    //         .collect::<Vec<_>>()
    //         .into(),
    // );

    // println!("{f_num}");

    Ok(())
}
