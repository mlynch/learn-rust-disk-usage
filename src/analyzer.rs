use std::{
    cell::{Cell, RefCell},
    fs::{self, metadata},
    rc::Rc,
};

use crate::{
    ctx::Context,
    stats::AnalyzerStats,
    utils::{bytes_to_human, is_hidden},
};

use colored::*;

#[derive(Clone)]
pub struct FileTreeNode {
    path: String,
    pub is_file: bool,
    mime_type: String,
    pub len: u64,
    children: RefCell<Vec<FileTreeNode>>,
}

impl FileTreeNode {
    fn new(path: String, is_file: bool, len: u64) -> FileTreeNode {
        let mut mime_str = String::from("");
        if let Some(mime) = mime_guess::from_path(path.clone()).first() {
            mime_str = mime.to_string();
        };

        FileTreeNode {
            path: path,
            mime_type: mime_str,
            is_file,
            len,
            children: RefCell::new(Vec::new()),
        }
    }

    fn push_child(&self, path: &str, is_file: bool, len: u64) -> Box<FileTreeNode> {
        let child = Box::new(FileTreeNode::new(String::from(path), is_file, len));

        self.children.borrow_mut().push((*child.as_ref()).clone());

        child
    }
}

pub struct Analyzer {
    total_bytes: Cell<u64>,
    tree: Rc<FileTreeNode>,
    stats: RefCell<AnalyzerStats>,
    files: RefCell<Vec<Box<FileTreeNode>>>,
}

impl Analyzer {
    pub fn new(root: String) -> Analyzer {
        let stats = RefCell::new(AnalyzerStats::new());

        Analyzer {
            tree: Rc::new(FileTreeNode::new(root, false, 0)),
            stats,
            total_bytes: Cell::new(0),
            files: RefCell::new(Vec::new()),
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

        for entry in entries {
            let path = &entry.path();

            if path.is_dir() {
                if !ctx.args.hidden && is_hidden(path) {
                    continue;
                }
                let path_str = path.to_str().unwrap();
                let new_child = node.push_child(path_str, true, 0);

                // let node = FileTreeNode::new(String::from(path_str));
                self.read_dir(ctx, &new_child)?;
            } else if path.is_file() {
                if !ctx.args.hidden && is_hidden(path) {
                    continue;
                }

                let len = metadata(path)?.len();

                self.total_bytes.set(self.total_bytes.get() + len);

                {
                    let path_str = Some(path.to_str().unwrap().clone());
                    let new_child = node.push_child(path_str.unwrap(), true, len);
                    self.files.borrow_mut().push(new_child);
                    self.stats
                        .borrow_mut()
                        .register_file(path_str.unwrap(), len);
                }
            }
        }

        Ok(())
    }

    pub fn get_by_type(&self, mime_type: &str) -> Vec<Box<FileTreeNode>> {
        let files = self.files.borrow();

        let filtered: Vec<Box<FileTreeNode>> = files
            .iter()
            .filter(|n| n.mime_type.contains(mime_type))
            .map(|n| n.clone())
            .collect();

        return filtered;

        /*
        let collected: RefCell<Vec<&FileTreeNode>> = RefCell::new(Vec::new());

        self._get_by_type(mime_type, &RefCell::new(self.tree.borrow()), &collected);
        return collected;
        */
    }

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

    pub fn print_report(&self, _ctx: &Context) {
        println!("{}", "Usage Report".bold().green());

        let print_type = |mime: &str| {
            let files = self.get_by_type(mime);
            println!("Total {}: {} ({})", mime, files.len(), bytes_to_human(files.iter().map(|f| f.len).sum::<u64>()));
        };


        print_type("image/");
        print_type("video/");
        print_type("application/zip");

        println!("{}", "Top files:".bold());
        self.stats.borrow().print_largest();

        println!("Total: {}", bytes_to_human(self.total_bytes.get()));
        println!("Total files: {}", self.files.borrow().len());
    }
}
