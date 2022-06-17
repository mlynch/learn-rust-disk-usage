use std::{thread, sync::{mpsc, Arc}};

use egui::mutex::RwLock;

use crate::{analyzer::Analyzer, ctx::Context, app::App};

mod ctx;
mod cli;
mod analyzer;
mod utils;
mod stats;
mod app;


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

    let current_file = Arc::new(RwLock::new(String::from("")));

    let producer_lock = current_file.clone();
    let app_lock = current_file.clone();

    thread::spawn(move|| {
        let analyzer = Analyzer::new(ctx.root.clone(), ctx.args.ignore.clone(), producer_lock);

        analyzer.analyze(&ctx).expect("Unable to read file or directory");
        analyzer.print_report(&ctx);

        // app.render_stats(&analyzer.stats.borrow());

        if ctx.args.delete_prompt {
            analyzer.prompt_delete();
        }
    });

    let app = App::new(app_lock);
}
