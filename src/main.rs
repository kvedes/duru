pub mod node;
use crate::node::Node;
use clap::Parser;
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    path: String,
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
    println!("{:?}", root);
}
