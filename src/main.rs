use anyhow::Result;
use combinator_parser::Combinator;

fn main() -> Result<()> {
    let input_str = "Qfghxy=f(gx)(hy)";

    let mut c = Combinator::parse(input_str)?;
    println!("{c}");
    c.abstraction_elimination();
    println!("{c}");

    Ok(())
}
