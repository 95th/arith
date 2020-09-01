use arith::{
    parser::Parser,
    syntax::{Context, Eval},
};
use std::rc::Rc;

fn main() {
    let s = r#"
        if iszero succ pred 0 { succ succ pred if true { succ 0 } else { 0 } } else { succ 0 }
    "#;

    let src = Rc::new(s.to_owned());

    let mut p = Parser::new(src.clone());
    let t = Rc::new(p.parse_expr());

    dbg!(&t);

    let eval = Eval::new(src);

    let ctx = &mut Context::default();
    let ty = eval.type_of(&t, ctx);
    println!("{:?}", ty);

    let ctx = &mut Context::default();

    let buf = &mut String::new();
    eval.print(&t, ctx, buf);
    println!("{}", buf);

    let t = eval.eval(&t, ctx);

    let buf = &mut String::new();
    eval.print(&t, ctx, buf);
    println!("{}", buf);
}
