use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::io::{self, BufRead, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!("Usage: toposort");
    eprintln!("Topological sort of a directed graph.");
    eprintln!();
    eprintln!("Reads edges from stdin (one pair per line, whitespace-separated).");
    eprintln!("Outputs nodes in topological order, one per line.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help    Show this help message");
    eprintln!("  -v, --version Show version");
}

fn main() {
if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") { jutils_core::print_version("toposort", env!("CARGO_PKG_VERSION")); std::process::exit(0); }
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("toposort {VERSION}");
        return;
    }

    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    let mut all_nodes: HashSet<String> = HashSet::new();

    let stdin = io::stdin();
    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            eprintln!("toposort: each line must have two space-separated nodes");
            std::process::exit(1);
        }
        let from = parts[0].to_string();
        let to = parts[1].to_string();

        all_nodes.insert(from.clone());
        all_nodes.insert(to.clone());

        adjacency.entry(from.clone()).or_default().push(to.clone());
        *in_degree.entry(to).or_insert(0) += 1;
        in_degree.entry(from).or_insert(0);
    }

    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(node, _)| node.clone())
        .collect();

    let mut sorted = Vec::new();

    while let Some(node) = queue.pop_front() {
        sorted.push(node.clone());
        if let Some(neighbors) = adjacency.get(&node) {
            for neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    if sorted.len() != all_nodes.len() {
        let remaining: Vec<&String> = all_nodes
            .iter()
            .filter(|n| !sorted.contains(*n))
            .collect();
        eprintln!(
            "toposort: cycle detected involving {} node(s): {}",
            remaining.len(),
            remaining
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        std::process::exit(1);
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    for node in &sorted {
        writeln!(out, "{node}").ok();
    }
}
