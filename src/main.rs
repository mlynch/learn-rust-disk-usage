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
mod pie_chart;

type LargeFile = (String, u64);
pub struct Scan {
    pub dir: String,
    pub started_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
    pub current_file: Option<String>,
    pub total_bytes: u64,
    pub largest_files: Box<Vec<LargeFile>>,
    pub total_music: u64,
    pub total_images: u64,
    pub total_videos: u64,
    pub total_documents: u64,
    pub total_binaries: u64,
    pub total_archives: u64,
    pub total_other: u64
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

    let scan_results = Arc::new(RwLock::new(Scan {
        dir: root,
        started_at: Local::now(),
        completed_at: None,
        current_file: Some(String::from("")),
        total_bytes: 0,
        largest_files: Box::new(vec![]),
        total_music: 0,
        total_images: 0,
        total_videos: 0,
        total_documents: 0,
        total_binaries: 0,
        total_archives: 0,
        total_other: 0
    }));

    let producer_lock = scan_results.clone();
    let app_lock = scan_results.clone();

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
