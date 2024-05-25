use std::borrow::BorrowMut;
use std::fmt;
use std::fs::{self, DirEntry};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
#[derive(Debug)]
pub enum DuruError {
    NotADir,
    NotAFile,
    IsLeaf,
    NotRoot,
    ExistingChildren,
    NoChildren,
    RootCantBeChild,
    FailedListExtraction,
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

    fn file_list_mutex(&mut self) -> Result<Arc<Mutex<Vec<DuruFile>>>, DuruError> {
        match self {
            Node::Root { children, .. } => {
                if children.is_some() {
                    let mut files = Arc::new(Mutex::new(Vec::new()));
                    file_list_recurse(self, Arc::clone(&files))
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
            Node::File { name, path, size } => Err(DuruError::NotRoot),
        }
    }

    pub fn file_list(&mut self) -> Result<Vec<DuruFile>, DuruError> {
        let flm = self.file_list_mutex();
        match flm {
            Ok(v) => Ok(Arc::try_unwrap(v)
                .map_err(|err| {
                    Err::<Arc<Mutex<Vec<DuruFile>>>, DuruError>(DuruError::FailedListExtraction)
                })
                .unwrap()
                .into_inner()
                .map_err(|err| Err::<Vec<DuruFile>, DuruError>(DuruError::FailedListExtraction))
                .unwrap()),
            Err(e) => Err(e),
        }
    }
}

fn file_list_recurse(
    node: &mut Node,
    file_list: Arc<Mutex<Vec<DuruFile>>>,
) -> Result<Arc<Mutex<Vec<DuruFile>>>, DuruError> {
    match node {
        Node::Root { children, path } => {
            if let Some(c) = children {
                for child in c.iter_mut() {
                    if let Node::Root { .. } = child {
                        return Err(DuruError::RootCantBeChild);
                    }
                    if let Node::Dir { .. } = child {
                        file_list_recurse(child, Arc::clone(&file_list))?;
                    }
                    if let Node::File { name, path, size } = child {
                        file_list.lock().unwrap().push(DuruFile::new(
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
            if let Some(c) = children {
                for child in c.iter_mut() {
                    if let Node::Root { .. } = child {
                        return Err(DuruError::RootCantBeChild);
                    }
                    if let Node::Dir { .. } = child {
                        file_list_recurse(child, Arc::clone(&file_list))?;
                    }
                    if let Node::File { name, path, size } = child {
                        file_list.lock().unwrap().push(DuruFile::new(
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
            file_list.lock().unwrap().push(DuruFile::new(
                name.to_string(),
                path.to_str().unwrap().to_string(),
                *size,
            ));
            Ok(file_list)
        }
    }
}

#[derive(Debug)]
pub struct DuruFile {
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
            Node::Root { .. } => Err(DuruError::NotAFile),
            Node::Dir { .. } => Err(DuruError::NotAFile),
        }
    }
}

pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug)]
pub struct DuruList {
    files: Vec<DuruFile>,
}

impl DuruList {
    pub fn new(files: Vec<DuruFile>) -> Self {
        DuruList { files }
    }

    pub fn sort_by_size(&mut self, sort_order: SortOrder) {
        match sort_order {
            SortOrder::Ascending => self.files.sort_by_key(|file| file.size),
            SortOrder::Descending => {
                self.files.sort_by_key(|file| file.size);
                self.files.reverse()
            }
        }
    }
}

impl fmt::Display for DuruList {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let max_chars = self
            .files
            .iter()
            .map(|f| f.name.chars().count())
            .max()
            .unwrap();

        for file in self.files.iter() {
            let indent = std::iter::repeat(" ")
                .take(max_chars - file.name.chars().count() + 1)
                .collect::<String>();
            write!(f, "{}{}{}\n", file.name, indent, file.size)?
        }
        Ok(())
    }
}
