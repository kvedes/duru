pub mod node;
use crate::node::{DuruList, Node, SortOrder};
use clap::Parser;
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    path: String,

    #[arg(long, default_value_t = 20)]
    head: u32,
}
fn main() {
    let args = Args::parse();

    let mut path = PathBuf::new();
    path.push(args.path);

    //let root_nodes = create_nodes(path);
    let mut root = Node::Root {
        children: None,
        path: path.clone(),
    };

    root.recurse();
    let file_list = root.file_list();

    let mut duru = DuruList::new(file_list.unwrap());
    duru.sort_by_size(SortOrder::Descending);
    //println!("{:?}", file_list);
    println!("{}", duru);
}
