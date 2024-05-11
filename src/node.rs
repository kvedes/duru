use std::fs::{self, DirEntry};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum DuruError {
    NotADir,
    IsLeaf,
    ExistingChildren,
}

#[derive(Clone, Debug)]
pub enum Node {
    File {
        name: String,
        path: PathBuf,
        size: u64,
    },
    Dir {
        name: String,
        path: PathBuf,
        size: Option<u64>,
        children: Option<Vec<Node>>,
    },
    Root {
        children: Option<Vec<Node>>,
        path: PathBuf,
    },
}

impl Node {
    pub fn recurse(&mut self) {
        match self {
            Node::Dir {
                name,
                path,
                size,
                children,
            } => {
                if children.is_none() {
                    let new_path = path.join(name);
                    *children = Some(Node::create_nodes(&new_path));
                } else {
                    ()
                }
                for child in children.as_mut().unwrap().iter_mut() {
                    child.recurse();
                }
            }
            Node::Root { children, path } => {
                if children.is_none() {
                    *children = Some(Node::create_nodes(path));
                } else {
                    ()
                }
                for child in children.as_mut().unwrap().iter_mut() {
                    child.recurse();
                }
            }
            Node::File { name, path, size } => (),
        }
    }

    pub fn create_nodes(path: &PathBuf) -> Vec<Node> {
        Node::to_nodes(Node::list_dir(path.clone().to_str().unwrap()), path)
    }

    pub fn list_dir(path: &str) -> Vec<DirEntry> {
        let paths = fs::read_dir(path).unwrap();
        paths
            .into_iter()
            .map(|e| e.unwrap())
            .collect::<Vec<DirEntry>>()
    }

    pub fn to_nodes(entries: Vec<DirEntry>, path: &PathBuf) -> Vec<Node> {
        entries
            .into_iter()
            .map(|node| {
                let metadata = node.metadata().unwrap();
                let is_dir = metadata.is_dir();
                if is_dir {
                    Node::Dir {
                        name: node.file_name().to_str().unwrap().to_string(),
                        path: path.clone(),
                        size: None,
                        children: None,
                    }
                } else {
                    Node::File {
                        name: node.file_name().to_str().unwrap().to_string(),
                        path: path.clone(),
                        size: metadata.size(),
                    }
                }
            })
            .collect()
    }
}
