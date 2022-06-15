use crate::{analyzer::Analyzer, ctx::Context};

mod ctx;
mod cli;
mod analyzer;
mod utils;
mod stats;


fn main() {
    let args = cli::get_args();

    let home = dirs::home_dir().unwrap();

    let mut root = args.dir.clone();

    if args.home {
        root = String::from(home.to_str().unwrap())
    }

    println!("Analyzing disk usage at {}", root);

    let ctx = Context {
        args,
        root: root.clone()
    };

    let analyzer = Analyzer::new(ctx.root.clone());

    analyzer.analyze(&ctx).expect("Unable to read file or directory");

    analyzer.print_report(&ctx);

    if ctx.args.delete_prompt {
        analyzer.prompt_delete();
    }
}
