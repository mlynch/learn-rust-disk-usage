use std::{
    cell::{Cell, RefCell},
    fs::{self, metadata, ReadDir},
    rc::Rc, path::Path, env::consts::OS, sync::{Arc},
    time::SystemTime
};

use chrono::{Local, DateTime};

use colored::*;

use dialoguer::{theme::ColorfulTheme, Select};

use egui::mutex::RwLock;
use glob::Pattern;
// use sysinfo::{ System };
use sysinfo::{DiskExt, System, SystemExt};

use crate::{
    stats::AnalyzerStats,
    utils::{bytes_to_human, is_hidden}, app::Scan
};

#[derive(Clone)]
pub struct FileTreeNode {
    path: String,
    pub is_file: bool,
    // mime_type: String,
    pub len: u64,
    // children: RefCell<Vec<FileTreeNode>>,
}

impl FileTreeNode {
    fn new(path: String, is_file: bool, len: u64) -> FileTreeNode {
        /*
        let mut mime_str = String::from("");
        if let Some(mime) = mime_guess::from_path(path.clone()).first() {
            mime_str = mime.to_string();
        };
        */

        FileTreeNode {
            path: path,
            // mime_type: mime_str,
            is_file,
            len,
            // children: RefCell::new(Vec::new()),
        }
    }

    /*
    fn push_child(&self, path: &str, is_file: bool, len: u64) -> Box<FileTreeNode> {
        let child = Box::new(FileTreeNode::new(String::from(path), is_file, len));

        self.children.borrow_mut().push((*child.as_ref()).clone());

        child
    }
    */
}

pub struct ScanSettings {
    pub dir: String,
    pub ignore: String,
    pub nlargest: u64,
    pub largebytes: u64,
    pub hidden: bool,
}


pub struct Analyzer<'a> {
    total_bytes: Cell<u64>,
    tree: Rc<FileTreeNode>,
    pub stats: RefCell<AnalyzerStats>,
    // files: RefCell<Vec<Box<FileTreeNode>>>,
    ignore_pattern: Pattern,
    settings: &'a ScanSettings,
    scan_results: Arc<RwLock<Scan>>
}

impl<'a> Analyzer<'a> {
    pub fn new(settings: &'a ScanSettings, scan_results: Arc<RwLock<Scan>>) -> Analyzer<'a> {
        let stats = RefCell::new(AnalyzerStats::new());

        Analyzer {
            tree: Rc::new(FileTreeNode::new(settings.dir.clone(), false, 0)),
            stats,
            total_bytes: Cell::new(0),
            // files: RefCell::new(Vec::new()),
            ignore_pattern: Pattern::new(settings.ignore.as_str()).expect("Unable to parse ignore glob pattern"),
            settings,
            scan_results
        }
    }

    pub fn analyze(&self) -> std::io::Result<()> {
        self.read_dir(self.tree.path.as_str());

        let mut w = self.scan_results.write();

        let stats = self.stats.borrow();

        (*w).current_file = None;
        (*w).largest_files = stats.largest_files.clone();
        (*w).completed_at = Some(Local::now());
        (*w).num_files = stats.num_files;
        (*w).total_archives = stats.total_archives;
        (*w).total_binaries = stats.total_binaries;
        (*w).total_documents = stats.total_documents;
        (*w).total_images = stats.total_images;
        (*w).total_music = stats.total_music;
        (*w).total_other = stats.total_other;
        (*w).total_videos = stats.total_videos;


        Ok(())
    }

    fn read_dir(&self, path: &str) -> u64 {
        let mut total_dir_usage: u64 = 0;

        let mut process_entries = |entries: ReadDir| {
            for entry in entries {
                let path = &entry.unwrap().path();

                if path.is_dir() && !self.should_skip(&path) {
                    if !self.settings.hidden && is_hidden(path) {
                        continue;
                    }
                    let path_str = path.to_str().unwrap();
                    // let new_child = node.push_child(path_str, true, 0);

                    // let node = FileTreeNode::new(String::from(path_str));
                    total_dir_usage += self.read_dir(path_str);

                    self.stats.borrow_mut().register_dir_usage(path, total_dir_usage);
                } else if path.is_file() {
                    if !self.settings.hidden && is_hidden(path) {
                        continue;
                    }

                    let mut w = self.scan_results.write();

                    (*w).current_file = Some(path.to_str().unwrap().to_string());

                    match metadata(path) {
                        Ok(meta) => {
                            let len = meta.len();

                            total_dir_usage += len;

                            (*w).total_bytes += len;

                            self.total_bytes.set(self.total_bytes.get() + len);

                            let path_str = Some(path.to_str().unwrap().clone());
                            // let new_child = node.push_child(path_str.unwrap(), true, len);
                            // self.files.borrow_mut().push(new_child);
                            self.stats
                                .borrow_mut()
                                .register_file(path_str.unwrap(), len, self.settings.nlargest, self.settings.largebytes);
                        },
                        Err(e) => println!("Unable to read file {} - {}", path.to_str().unwrap(), e)
                    }
                }
            }

        };

        match fs::read_dir(path) {
            Ok(entries) => {
                process_entries(entries);
            },
            Err(e) => {
                println!("Unable to read directory {} - {}", path, e);
            }
        }

        total_dir_usage
    }

    /*
    pub fn get_by_type(&self, mime_type: &str) -> Vec<Box<FileTreeNode>> {
        let files = self.files.borrow();

        let filtered: Vec<Box<FileTreeNode>> = files
            .iter()
            .filter(|n| n.mime_type.contains(mime_type))
            .map(|n| n.clone())
            .collect();

        return filtered;
    }
    */
    /*
    fn _get_by_type<'a>(&self, mime_type: &str, node: &RefCell<&'a FileTreeNode>, collected: &RefCell<Vec<&'a FileTreeNode>>) {
        let children = node.borrow().children.borrow();

        let iter: Vec<&FileTreeNode> = children.iter().map(|n| n).collect();

        iter.
            iter().
            filter(|n| n.mime_type == mime_type).
            for_each(|n| collected.borrow_mut().push(n.clone()));

        for ele in children.iter() {
            if !ele.is_file {
                self._get_by_type(mime_type, &RefCell::new(ele), collected);
            }
        }
    }
    */

    fn should_skip(&self, path: &Path) -> bool {
        // Skip symlinks
        if path.is_symlink() {
            return true
        }

        if self.ignore_pattern.matches(path.to_str().unwrap()) {
            println!("Skipping ignored path: {:?}", path.to_str());
            return true
        }

        if OS == "macos" {
            if let Some(ospath) = path.file_name() {
                if let Some(filename) = ospath.to_str() {
                    if filename.contains(".app") {
                        return true
                    }
                }
            }
        }

        return false
    }

    pub fn print_report(&self) {
        println!("{}", "\n-- Usage Report --\n".bright_yellow());

        let mut sys = System::new_all();
        sys.refresh_all();

        println!("{}", "Totals:".bright_green());
        println!("  Disk usage: {}", bytes_to_human(self.total_bytes.get()));

        println!("");

        println!("{}", "Current disk usage:".bright_green());
        for disk in sys.disks() {
            let p = (disk.available_space() as f64 / disk.total_space() as f64) * 100.0;
            println!(
                "  {} ({} free ({:.2}%) , {} total)",
                disk.name().to_str().unwrap(),
                bytes_to_human(disk.available_space()),
                p,
                bytes_to_human(disk.total_space())
            );
        }

        println!("");

        let print_type = |type_name: &str, len: u64| {
            println!("  {}: {}", type_name, bytes_to_human(len));
        };

        let stats = self.stats.borrow();
        println!("{}", "File types:".bright_green());
        print_type("Images", stats.total_images);
        print_type("Videos", stats.total_videos);
        print_type("Music", stats.total_music);
        print_type("Documents", stats.total_documents);
        print_type("Archives", stats.total_archives);
        print_type("Binaries", stats.total_binaries);
        print_type("Other", stats.total_other);

        println!("");

        println!("{}", "Top files:".bright_green());
        self.stats.borrow().print_largest();
    }

    pub fn prompt_delete(&self) {
        let mut total_deleted: u64 = 0;

        let selections = &[
            "Keep",
            "Exit",
            "Delete (trash)",
            "Delete (force)"
        ];
        for file in self.stats.borrow().get_largest() {
            println!("");
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Delete {} ({})?", file.0, bytes_to_human(file.1)))
                .default(0)
                .items(&selections[..])
                .interact_opt()
                .unwrap();

            if let Some(selection) = selection {
                if selection == 1 {
                    // Exit on exit
                    break
                }

                if selection == 2 {
                    println!("Deleting {}", file.0);
                    match trash::delete(&file.0) {
                        Ok(_) => {
                            println!("Deleted!");
                            total_deleted += file.1;
                        },
                        Err(e) => println!("Unable to delete: {}", e)
                    }
                } else if selection == 3 {
                    println!("Deleting (force) {}", file.0);
                    match fs::remove_file(&file.0) {
                        Ok(_) => {
                            println!("Deleted!");
                            total_deleted += file.1;
                        },
                        Err(e) => println!("Unable to delete: {}", e)
                    }
                }
            }
        }

        println!("");
        println!("Reclaimed {} of disk space", bytes_to_human(total_deleted));
    }
}

#[cfg(test)]
mod tests {
    use glob::Pattern;

    #[test]
    fn pattern_match() {
        let pattern = Pattern::new("**/node_modules").expect("Unable to parse ignore glob pattern");

        assert_eq!(pattern.matches("/Users/max/git/project/node_modules"), true);
    }
}

