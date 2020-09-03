use arith::{
    parser::Parser,
    syntax::{Context, Eval, TyContext},
};
use std::rc::Rc;

fn main() {
    let s = r#"
        if if true { iszero 0 } else { iszero succ 0 } { succ succ pred if true { succ 0 } else { succ succ 0 } } else { succ 0 }
    "#;

    let src = Rc::new(s.to_owned());

    let mut p = Parser::new(src.clone());
    let t = Rc::new(p.parse_expr());

    let eval = Eval::new(src);

    let ctx = &mut Context::default();
    let ty_ctx = &mut TyContext::new();
    let ty = eval.type_of(&t, ctx, ty_ctx);
    ty_ctx.print(ty);

    let ctx = &mut Context::default();

    let buf = &mut String::new();
    eval.print(&t, ctx, buf);
    println!("{}", buf);

    let t = eval.eval(&t, ctx);

    let buf = &mut String::new();
    eval.print(&t, ctx, buf);
    println!("{}", buf);
}
