use arith::{
    parser::Parser,
    syntax::{Context, Eval, TyContext},
};
use std::rc::Rc;

fn main() {
    let s = r#"
        lambda a: bool -> lambda a: nat -> lambda a: nat -> iszero 0
    "#;

    let src = Rc::new(s.to_owned());
    let tyctx = &mut TyContext::new();

    let mut p = Parser::new(src.clone());
    let t = Rc::new(p.parse_expr(tyctx));

    let eval = Eval::new(src);

    let ctx = &mut Context::default();
    let ty = eval.type_of(&t, ctx, tyctx);
    tyctx.print(ty);

    let ctx = &mut Context::default();

    let buf = &mut String::new();
    eval.print(&t, ctx, buf);
    println!("{}", buf);

    let t = eval.eval(&t, ctx);

    let ctx = &mut Context::default();
    let buf = &mut String::new();
    eval.print(&t, ctx, buf);
    println!("{}", buf);
}
