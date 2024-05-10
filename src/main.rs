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
    // pub fn append(&mut self, child: Node) -> Result<(), DuruError> {
    //     match (self, child) {
    //         (
    //             Node::Dir {
    //                 name,
    //                 path,
    //                 size,
    //                 children,
    //             },
    //             Node::File {
    //                 name: _n,
    //                 path: _p,
    //                 size: _s,
    //             },
    //         ) => Ok(children.as_mut().unwrap().push(child.clone())),
    //         (
    //             Node::Dir {
    //                 name,
    //                 path,
    //                 size,
    //                 children,
    //             },
    //             Node::Dir {
    //                 name: _n,
    //                 path: _p,
    //                 size: _s,
    //                 children: _c,
    //             },
    //         ) => Ok(children.unwrap().push(child)),
    //         (
    //             Node::Root { children, path },
    //             Node::File {
    //                 name: _n,
    //                 path: _p,
    //                 size: _s,
    //             },
    //         ) => Ok(children.unwrap().push(child)),
    //         (
    //             Node::Root { children, path },
    //             Node::Dir {
    //                 name: _n,
    //                 path: _p,
    //                 size: _s,
    //                 children: _c,
    //             },
    //         ) => Ok(children.unwrap().push(child)),
    //         _ => Err(DuruError::NotADir),
    //     }
    // }

    // pub fn create_children(&mut self) -> Result<(), DuruError> {
    //     match self {
    //         Node::Dir {
    //             name,
    //             path,
    //             size,
    //             children,
    //         } => {
    //             children.as_mut().unwrap().append(&mut create_nodes(path));
    //             Ok(())
    //         }
    //         Node::Root { children, path } => {
    //             children.as_mut().unwrap().append(&mut create_nodes(path));
    //             Ok(())
    //         }
    //         Node::File { name, path, size } => Err(DuruError::IsLeaf),
    //     }
    // }

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
                    *children = Some(create_nodes(&new_path));
                } else {
                    ()
                }
                for child in children.as_mut().unwrap().iter_mut() {
                    child.recurse();
                }
            }
            Node::Root { children, path } => {
                if children.is_none() {
                    *children = Some(create_nodes(path));
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

    // pub fn append_children(&mut self, children: Vec<Node>) -> Result<(), DuruError> {
    //     match self {
    //         Node::Dir {
    //             name,
    //             path,
    //             size,
    //             children: c,
    //         } => {
    //             if c.is_none() {
    //                 *c = Some(children);
    //                 Ok(())
    //             } else {
    //                 Err(DuruError::ExistingChildren)
    //             }
    //         }
    //         Node::Root { children: c, path } => match c {
    //             None => {
    //                 c = &mut Some(children);
    //                 Ok(())
    //             }
    //             Some(v) => Err(DuruError::ExistingChildren),
    //         },
    //         Node::File { name, path, size } => Err(DuruError::IsLeaf),
    //     }
    // }

    pub fn get_dirs(&self) -> Result<&Vec<Node>, DuruError> {
        match self {
            Node::Dir {
                name,
                path,
                size,
                children,
            } => Ok(children.as_ref().unwrap()),
            Node::Root { children, path } => Ok(children.as_ref().unwrap()),
            Node::File { name, path, size } => Err(DuruError::IsLeaf),
        }
    }

    // pub fn get_dirs_mut<'a>(&'a mut self) -> Result<&'a mut Vec<Node>, DuruError> {
    //     match self {
    //         Node::Dir {
    //             name,
    //             path,
    //             size,
    //             children,
    //         } => Ok(children.as_mut().unwrap()),
    //         Node::Root { children, path } => Ok(children.as_mut().unwrap()),
    //         Node::File { name, path, size } => Err(DuruError::IsLeaf),
    //     }
    // }
}

fn create_nodes(path: &PathBuf) -> Vec<Node> {
    to_nodes(list_dir(path.clone().to_str().unwrap()), path)
}

fn list_dir(path: &str) -> Vec<DirEntry> {
    let paths = fs::read_dir(path).unwrap();
    paths
        .into_iter()
        .map(|e| e.unwrap())
        .collect::<Vec<DirEntry>>()
}

fn to_nodes(entries: Vec<DirEntry>, path: &PathBuf) -> Vec<Node> {
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

// pub fn create_children(node: Node) -> Result<Vec<Node>, DuruError> {
//     match node {
//         Node::Dir {
//             name,
//             path,
//             size,
//             children,
//         } => Ok(create_nodes(path)),
//         Node::Root { children, path } => Ok(create_nodes(path)),
//         Node::File { name, path, size } => Err(DuruError::IsLeaf),
//     }
// }

fn main() {
    let mut path = PathBuf::new();
    path.push("/home/mat/work/duru");

    //let root_nodes = create_nodes(path);
    let mut root = Node::Root {
        children: None,
        path: path.clone(),
    };

    root.recurse();
    println!("{:?}", root);
    //println!("{:?}", path);
    let nodes = create_nodes(&path);
    println!("{:?}", nodes);
    // let mut level = vec![&mut root];

    // //let mut current_dirs = root.get_dirs().unwrap();
    // loop {
    //     level.iter_mut().map(|dir| create_children(dir));
    //     level = level
    //         .iter_mut()
    //         .map(|dir| dir.get_dirs_mut())
    //         .filter(|r| r.is_ok())
    //         .map(|r| r.unwrap())
    //         .flatten()
    //         .collect();

    //     if level.len() == 0 {
    //         break;
    //     }
    //     // let tmp_dirs: Vec<Node> = Vec::new();
    //     // for dir in current_dirs.iter_mut() {
    //     //     dir.create_children().unwrap();
    //     //     for d in dir.get_dirs().unwrap() {
    //     //         tmp_dirs.push(d.clone());
    //     //     }
    //     // }
    //     // current_dirs = tmp_dirs;
    // }
    // println!("{:?}", level);

    // for path in paths {
    //     path
    //     println!("Name: {}", path.unwrap().path().display())
    // }
}
