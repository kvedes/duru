use std::fs::{self, DirEntry};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum DuruError {
    NotADir,
    NotAFile,
    IsLeaf,
    NotRoot,
    ExistingChildren,
    NoChildren,
    RootCantBeChild,
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

    pub fn file_list(&self) -> Result<&mut Vec<DuruFile>, DuruError> {
        match self {
            Node::Root { children, path } => {
                if children.is_some() {
                    self.file_list_recurse(&mut Vec::new())
                } else {
                    Err(DuruError::NoChildren)
                }
            }
            Node::Dir {
                name,
                path,
                size,
                children,
            } => Err(DuruError::NotRoot),
            Node::File { name, path, size } => Err(DuruError::IsLeaf),
        }
    }

    fn file_list_recurse(
        &self,
        file_list: &mut Vec<DuruFile>,
    ) -> Result<&mut Vec<DuruFile>, DuruError> {
        match self {
            Node::Root { children, path } => {
                if let Some(children) = children.as_mut() {
                    for child in children.iter_mut() {
                        if let Node::Root { .. } = child {
                            return Err(DuruError::RootCantBeChild);
                        }
                        if let Node::Dir { .. } = child {
                            self.file_list_recurse(file_list)?;
                        }
                        if let Node::File { name, path, size } = child {
                            file_list.push(DuruFile::new(
                                name.to_string(),
                                path.to_str().unwrap().to_string(),
                                *size,
                            ));
                        }
                    }
                }
                Ok(file_list)
            }
            Node::Dir {
                name,
                path,
                size,
                children,
            } => {
                if let Some(children) = children.as_mut() {
                    for child in children.iter_mut() {
                        if let Node::Root { .. } = child {
                            return Err(DuruError::RootCantBeChild);
                        }
                        if let Node::Dir { .. } = child {
                            self.file_list_recurse(file_list)?;
                        }
                        if let Node::File { name, path, size } = child {
                            file_list.push(DuruFile::new(
                                name.to_string(),
                                path.to_str().unwrap().to_string(),
                                *size,
                            ));
                        }
                    }
                }
                Ok(file_list)
            }
            Node::File { name, path, size } => {
                file_list.push(DuruFile::new(
                    name.to_string(),
                    path.to_str().unwrap().to_string(),
                    *size,
                ));
                Ok(file_list)
            }
        }
    }
}

struct DuruFile {
    name: String,
    path: String,
    size: u64,
}

impl DuruFile {
    pub fn new(name: String, path: String, size: u64) -> Self {
        DuruFile { name, path, size }
    }

    pub fn from_node(node: Node) -> Result<Self, DuruError> {
        match node {
            Node::File { name, path, size } => Ok(DuruFile::new(
                name,
                path.to_str().unwrap().to_string(),
                size,
            )),
            Node::Root { children, path } => Err(DuruError::NotAFile),
            Node::Dir {
                name,
                path,
                size,
                children,
            } => Err(DuruError::NotAFile),
        }
    }
}

//pub fn files_to_list(root: Node) -> Vec<DuruFile> {}
