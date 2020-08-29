use arith::{
    untyped::{Context, TermKind::*},
    U,
};

fn main() {
    let t = U![Abs {
        name: String::from("a"),
        term: U![App {
            target: U![Abs {
                name: String::from("b"),
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
}
