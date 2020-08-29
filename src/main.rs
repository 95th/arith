use arith::{
    untyped::{Context, TermKind::*, Ty},
    U,
};
use std::rc::Rc;

fn main() {
    let t = U![Abs {
        name: String::from("a"),
        ty: Rc::new(Ty::Bool),
        term: U![App {
            target: U![Abs {
                name: String::from("b"),
                ty: Rc::new(Ty::Bool),
                term: U![Var { idx: 1, len: 2 }],
            }],
            val: U![Var { idx: 0, len: 2 }],
        }]
    }];

    let ctx = &mut Context::default();
    let t = t.eval(ctx);

    let buf = &mut String::new();
    t.print(ctx, buf);
    println!("{}", buf);

    let ctx = &mut Context::default();
    let t = t.type_of(ctx);
    println!("{:?}", t);
}
