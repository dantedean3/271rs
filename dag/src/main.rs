use std::collections::HashMap;
use std::io::{self, BufRead};

use dag::{Node, Graph, depth, height};

fn main() {
    let mut graph: Graph = HashMap::new();
    let stdin = io::stdin();

    // Read edges like "C151:C280"
    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(l) => l.trim().to_string(),
            Err(_) => continue,
        };

        if line.is_empty() {
            break;
        }

        let (from, to) = match line.split_once(':') {
            Some((a, b)) => (a.trim(), b.trim()),
            None => continue,
        };

        // Ensure destination exists
        let dest_node = graph
            .entry(to.to_string())
            .or_insert_with(|| Node::new(to));
        dest_node.add_prereq(from);

        // Ensure source exists too
        graph
            .entry(from.to_string())
            .or_insert_with(|| Node::new(from));
    }

    // Compute and print required values
    dbg!(depth("C152", &graph));
    dbg!(depth("C371", &graph));
    dbg!(height("C152", &graph));
    dbg!(height("C371", &graph));
}
