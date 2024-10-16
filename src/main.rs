use anyhow::Result;
use combinator_parser::{AbstractionElimination, Combinator, CombinatorContext};

fn prelude(context: &mut CombinatorContext) {
    vec![
        "M=SII",
        "Y=BM(CBM)",
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

    let mut ap = Combinator::parse("{Ap}anfx={IsZero}nx(f(a({Prev}n)fx))")?;
    ap.abstraction_elimination()
        .context_substitution(&context)
        .reduce_expression();
    println! {"{ap}"}
    context.register_loose(ap)?;

    let mut apply = Combinator::parse("{Apply}=Y{Ap}")?;
    apply
        .abstraction_elimination()
        .context_substitution(&context)
        .reduce_expression();


    Ok(())
}
