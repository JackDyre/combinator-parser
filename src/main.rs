use anyhow::Result;
use combinator_parser::Combinator;

fn main() -> Result<()> {
    let input_str = "Qfgx=f(gx)(gx)";

    let c = Combinator::parse(input_str)?;

    println!("{c:?}");

    Ok(())
}
