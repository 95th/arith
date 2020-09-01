use arith::{parser::Parser, syntax::Context};
use std::rc::Rc;

fn main() {
    let s = r#"
        if iszero pred succ 0 { 0 } else { succ 0 }
    "#;

    let mut p = Parser::new(Rc::new(s.to_owned()));
    let t = Rc::new(p.parse_expr());

    let ctx = &mut Context::default();

    let buf = &mut String::new();
    t.print(ctx, buf);
    println!("{}", buf);

    let t = t.eval(ctx);

    let buf = &mut String::new();
    t.print(ctx, buf);
    println!("{}", buf);

    let ctx = &mut Context::default();
    let t = t.type_of(ctx);
    println!("{:?}", t);
}
