macro_rules! quit {
    () => {
        std::process::exit(1);
    };
    ($msg:expr) => {{
        println!($msg);
        quit!();
    }};
    ($msg:expr,) => {
        quit!($msg);
    };
    ($fmt:expr, $($arg:tt)+) => {{
        println!($fmt, $($arg)+);
        quit!();
    }};
}

#[macro_use]
extern crate lazy_static;

pub mod initial;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod untyped;
