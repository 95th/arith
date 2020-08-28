use arith::{
    untyped::{Context, TermKind::*},
    U,
};

fn main() {
    let t = U![Abstraction {
        name: String::from("a"),
        term: U![Application {
            target: U![Abstraction {
                name: String::from("b"),
                term: U![Variable { idx: 1, len: 2 }],
            }],
            val: U![Variable { idx: 0, len: 2 }],
        }]
    }];

    let ctx = &mut Context::default();
    let t = t.eval(ctx);

    let buf = &mut String::new();
    t.print(ctx, buf);
    println!("{}", buf);
}
