pub mod node;
use crate::node::{DuruList, Node, SortOrder};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,

    #[arg(long, default_value_t = 20)]
    head: usize,

    #[arg(short, long, action)]
    full: bool,
}
fn main() {
    let args = Args::parse();

    let mut path = PathBuf::new();
    path.push(args.path);

    let mut root = Node::Root {
        children: None,
        path: path.clone(),
    };

    root.recurse();
    let file_list = root.file_list();

    let mut duru = DuruList::new(file_list.unwrap());
    duru.sort_by_size(SortOrder::Descending);
    let duru_list = duru.head(args.head);

    if args.full {
        duru_list.print_path_size();
    } else {
        duru_list.print_name_size();
    }
}
