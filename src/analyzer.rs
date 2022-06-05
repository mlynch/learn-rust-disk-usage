use std::{fs::{File, self, ReadDir, metadata}, cell::{Cell, RefCell}, rc::Rc};

use priority_queue::PriorityQueue;

use crate::{ctx::Context, utils::is_hidden};

struct FileTreeNode {
    path: String,
    mime_type: String,
    children: Vec<FileTreeNode>
}

impl FileTreeNode {
    fn new(path: String) -> FileTreeNode {
        let mut mime_str = String::from("");
        if let Some(mime) = mime_guess::from_path(path.clone()).first() {
            mime_str = mime.to_string();
        };

        FileTreeNode {
            path: path,
            mime_type: mime_str,
            children: Vec::new()
        }
    }

    fn push_child(&self, path: &str) {
    }
}

pub struct Analyzer {
    tree: Rc<FileTreeNode>,
    stats: RefCell<AnalyzerStats>
}

pub struct AnalyzerStats {
    largest_files: PriorityQueue<String, u64>
}

impl AnalyzerStats {
    pub fn register_file(&mut self, path_str: &str, len: u64) {
        self.largest_files.push(path_str.to_owned(), len);
    }

    pub fn print_largest(&self) {
        println!("Largest: {}", self.largest_files.len());

        let largest_files = self.largest_files.clone();

        let sorted = largest_files.into_sorted_iter();

        for (path, len) in sorted.take(100) {
            println!("{} ({})", path, len);
        }
    }
}

impl Analyzer {
    pub fn new(root: String) -> Analyzer {

        let stats = RefCell::new(AnalyzerStats {
            largest_files: PriorityQueue::with_capacity(100)
        });

        Analyzer {
            tree: Rc::new(FileTreeNode::new(root)),
            stats
        }
    }

    pub fn analyze(&self, ctx: &Context) -> std::io::Result<()> {
        self.read_dir(ctx, self.tree.as_ref())?;

        Ok(())
    }

    fn read_dir(&self, ctx: &Context, node: &FileTreeNode) -> std::io::Result<()> {
        /*
        {
            let entries = &mut fs::read_dir(node.path).unwrap_or();
            println!("Reading {} entries for dir: {}", entries.count(), node.path);
        }
        */

        let entries = fs::read_dir(node.path.as_str())?.filter_map(Result::ok);

        println!("Got these entries {}", node.path);

        for entry in entries {
            let path = &entry.path();

            if path.is_dir() {
                if !ctx.args.hidden && is_hidden(path) {
                    continue
                }
                let path_str = path.to_str().unwrap();
                node.push_child(path_str);
                println!("{}", path_str);

                let node = FileTreeNode::new(String::from(path_str));
                self.read_dir(ctx, &node)?;
            } else if path.is_file() {
                if !ctx.args.hidden && is_hidden(path) {
                    continue
                }

                let mut path_str: Option<&str> = Option::None;
                {
                    path_str = Some(path.to_str().unwrap().clone());
                    node.push_child(path_str.unwrap());
                }
                let len = metadata(path)?.len();

                //let mut stats = &self.stats;

                //Rc::get_mut(self.stats).register_file(path_str.unwrap(), len);
                self.stats.borrow_mut().register_file(path_str.unwrap(), len);

                println!("{} ({})", path_str.unwrap(), len);
            }
        }

        Ok(())
    }

    pub fn print_report(&self, ctx: &Context) {
        println!("Got largest files: ");

        self.stats.borrow().print_largest();
    }
}