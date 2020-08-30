use arith::{
    parser::Parser,
    untyped::{Context, TermKind::*, Ty},
    U,
};
use std::rc::Rc;

fn main() {
    let s = r#"
        if true {
            if true {
                true
            } else {
                false
            }
        } else {
            false
        }
    "#;
    let mut p = Parser::new(Rc::new(s.to_owned()));
    let e = p.parse_expr();
    println!("{:#?}", e);
}

pub fn main2() {
    let t = U![Fun {
        name: String::from("a"),
        ty: Rc::new(Ty::Arrow {
            from: Rc::new(Ty::Bool),
            to: Rc::new(Ty::Bool)
        }),
        term: U![Call {
            callee: U![Fun {
                name: String::from("b"),
                ty: Rc::new(Ty::Bool),
                term: U![Var { idx: 1, len: 2 }],
            }],
            arg: U![Var { idx: 0, len: 2 }],
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
