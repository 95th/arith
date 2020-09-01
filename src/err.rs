use crate::span::Span;
use std::rc::Rc;

#[macro_export]
macro_rules! quit {
    () => {
        std::process::exit(1);
    };
    ($src:expr, $span:expr, $msg:expr) => {{
        let msg = $msg.to_owned();
        let d = $crate::err::Diagnostic::new($src, msg, $span);
        d.report();
        quit!();
    }};
    ($src:expr, $span:expr, $msg:expr,) => {
        quit!($src, $span, $msg);
    };
    ($src:expr, $span:expr, $fmt:expr, $($arg:tt)+) => {{
        quit!($src, $span, format!($fmt, $($arg)+))
    }};
}

pub struct Diagnostic {
    src: Rc<String>,
    msg: String,
    span: Span,
}

impl Diagnostic {
    pub fn new(src: &Rc<String>, msg: String, span: Span) -> Self {
        Self {
            src: src.clone(),
            msg,
            span,
        }
    }

    pub fn report(&self) {
        if self.span.hi > self.src.len() {
            self.report_out_of_bounds()
        } else {
            self.report_in_bounds()
        }
    }

    fn report_in_bounds(&self) {
        let start = self.line_start(self.span.lo);
        let end = self.line_end(self.span.hi);
        let line_count = self.line_count(start, end);
        if line_count == 1 {
            println!("{}", &self.src[start..end]);
            let mut buf = String::with_capacity(end - start);
            for _ in start..self.span.lo {
                buf.push(' ');
            }
            for _ in self.span.lo..self.span.hi {
                buf.push('^');
            }
            buf.push(' ');
            buf.push_str(&self.msg);
            println!("{}", buf);
        } else {
            println!("{}", &self.src[start..end]);
            println!(" {}", self.msg);
        }
    }

    fn report_out_of_bounds(&self) {
        let start = self.line_start(self.span.lo);
        let end = self.line_end(self.span.hi);
        println!("{}", &self.src[start..end]);
        let mut buf = String::with_capacity(end - start);
        for _ in start..self.src.len() {
            buf.push(' ');
        }
        buf.push_str("^ ");
        buf.push_str(&self.msg);
        println!("{}", buf);
    }

    fn line_start(&self, from: usize) -> usize {
        let from = from.min(self.src.len() - 1);
        self.src[..from]
            .rfind('\n')
            .map(|n| n + 1)
            .unwrap_or_default()
    }

    fn line_end(&self, from: usize) -> usize {
        let from = from.min(self.src.len() - 1);
        self.src[from..]
            .find('\n')
            .map(|n| from + n)
            .unwrap_or(self.src.len())
    }

    fn line_count(&self, start: usize, end: usize) -> usize {
        self.src[start..end].bytes().filter(|&n| n == b'\n').count() + 1
    }
}
