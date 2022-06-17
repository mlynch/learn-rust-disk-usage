use std::{thread, sync::{Arc}};

use chrono::{Local, DateTime};
use egui::mutex::RwLock;

use crate::{analyzer::Analyzer, ctx::Context, app::App};

mod ctx;
mod cli;
mod analyzer;
mod utils;
mod stats;
mod app;

type LargeFile = (String, u64);
pub struct Scan {
    pub dir: String,
    pub started_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
    pub current_file: Option<String>,
    pub total_bytes: u64,
    pub largest_files: Box<Vec<LargeFile>>,
}

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

    let current_file = Arc::new(RwLock::new(Scan {
        dir: root,
        started_at: Local::now(),
        completed_at: None,
        current_file: Some(String::from("")),
        total_bytes: 0,
        largest_files: Box::new(vec![])
    }));

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
