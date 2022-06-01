use crate::{analyzer::Analyzer, ctx::Context};

mod ctx;
mod analyzer;


fn main() {
    let ctx = Context {
        root: "/"
    };

    let analyzer = Analyzer::new(ctx.root);

    analyzer.analyze(&ctx);

    println!("Hello, world!");
}
