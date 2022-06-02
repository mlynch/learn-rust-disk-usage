use std::fs::{File, self, ReadDir, metadata};

use crate::{ctx::Context, utils::is_hidden};

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
        self.read_dir(ctx, &self.tree)?;

        Ok(())
    }

    fn read_dir(&self, ctx: &'a Context, node: &FileTreeNode) -> std::io::Result<()> {
        {
            let entries = &mut fs::read_dir(node.path)?;
            println!("Reading {} entries for dir: {}", entries.count(), node.path);
        }

        let entries = fs::read_dir(node.path)?;

        for entry in entries {
            let entry = entry?;
            let path = &entry.path();

            if path.is_dir() {
                if !ctx.args.hidden && is_hidden(path) {
                    continue
                }
                let path_str = path.to_str().unwrap();
                node.push_child(path_str);
                println!("{}", path_str);

                let node = FileTreeNode::new(path_str);
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
                println!("{} ({})", path_str.unwrap(), len);
            }
        }

        Ok(())
    }
}