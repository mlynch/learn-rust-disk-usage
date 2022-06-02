use crate::{analyzer::Analyzer, ctx::Context};

mod ctx;
mod cli;
mod analyzer;
mod utils;


fn main() {
    let args = cli::get_args();

    let home = dirs::home_dir().unwrap();

    let mut root = "/";

    if args.home {
        root = home.to_str().unwrap()
    }

    println!("Analyzing disk usage at {}", root);

    let ctx = Context {
        args,
        root
    };

    let analyzer = Analyzer::new(ctx.root);

    analyzer.analyze(&ctx).expect("Unable to read file or directory");

    println!("Hello, world!");
}
