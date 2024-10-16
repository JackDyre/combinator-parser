use anyhow::Result;
use combinator_parser::{AbstractionElimination, Combinator, CombinatorContext, Expression};

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
        "{ApplyPrime}anfx={IsZero}nx(f(a({Prev}n)fx))",
        "{Apply}=Y{ApplyPrime}",
        "{Add}nm={Apply}n{Next}m",
        "{Mult}nm={Apply}n({Add}m)0",
        "{NextFibPair}p={Pair}({Second}p)({Add}({First}p)({Second}p))",
        "{Fib}n={First}({Apply}n{NextFibPair}({Pair}01))",
    ]
    .into_iter()
    .for_each(|p| {
        let mut c = Combinator::parse(p).unwrap();
        c.abstraction_elimination().context_substitution(context);
        context.register_loose(c).unwrap();
    });
}

fn main() -> Result<()> {
    let mut context = CombinatorContext::new();
    prelude(&mut context);

    let mut e = Expression::parse(&number_to_string(8))?;
    println!("{e}");
    e.context_substitution(&context).reduce_expression();
    println!("{e}");

    Ok(())
}

fn number_to_string(n: i32) -> String {
    if n == 0 {
        return "{Apply}({Fib}(0))fx".to_string();
    }

    let mut next_expr = "0".to_string();

    // Generate the "Next" expression
    for _ in 0..n {
        next_expr = format!("{{Next}}({})", next_expr);
    }

    // Wrap the "Next" expression in "{Fib}(...)"
    let fib_expr = format!("{{Fib}}({})", next_expr);

    // Wrap everything in "{Apply}(...)fx"
    let apply_expr = format!("{{Apply}}({})fx", fib_expr);

    apply_expr
}
