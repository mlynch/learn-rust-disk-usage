use std::fs::{File, self, ReadDir};

use crate::ctx::Context;

struct FileTreeNode<'a> {
    path: &'a str,
    children: Vec<FileTreeNode<'a>>
}

impl<'a> FileTreeNode<'a> {
    fn new(path: &str) -> FileTreeNode {
        FileTreeNode {
            path,
            children: Vec::new()
        }
    }

    fn push_child(&self, path: &str) {
    }
}

pub struct Analyzer<'a> {
    tree: FileTreeNode<'a>
}

impl<'a> Analyzer<'a> {
    pub fn new(root: &'a str) -> Analyzer<'a> {
        Analyzer {
            tree: FileTreeNode::new(root)
        }
    }

    pub fn analyze(&self, ctx: &'a Context) -> std::io::Result<()> {
        self.read_dir(&self.tree)?;

        Ok(())
    }

    fn read_dir(&self, node: &FileTreeNode) -> std::io::Result<()> {
        let entries = fs::read_dir(node.path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();


            if path.is_dir() {
                node.push_child(path.to_str().unwrap());
                println!("{}", path.to_str().unwrap());

                let node = FileTreeNode::new(path.to_str().unwrap());
                self.read_dir(&node)?;
            } else if path.is_file() {
                node.push_child(path.to_str().unwrap());
                println!("{}", path.to_str().unwrap());
            }
        }

        Ok(())
    }
}