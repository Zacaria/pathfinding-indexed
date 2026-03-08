/// Example demonstrating how to use the BMSSP-based shortest path algorithm.
///
/// This example uses a weighted directed graph stored as an adjacency list. It shows the
/// index-only API using `IndexedGraphMap` and `IndexedGraph`.
use pathfinding_indexed::IndexedGraphMap;
use std::collections::HashMap;

fn main() {
    // Create a weighted directed graph using an adjacency list.
    // Each node maps to a list of (neighbor, weight) pairs.
    let graph: HashMap<&str, Vec<(&str, u32)>> = [
        ("A", vec![("B", 4), ("C", 2)]),
        ("B", vec![("C", 1), ("D", 5)]),
        ("C", vec![("D", 8), ("E", 10)]),
        ("D", vec![("E", 2)]),
        ("E", vec![]),
        // A disconnected node (unreachable from "A").
        ("F", vec![]),
    ]
    .into_iter()
    .collect();

    let mapped = IndexedGraphMap::from_nodes_and_successors(["A"], |node| {
        graph.get(node).cloned().unwrap_or_default()
    });

    let start_idx = mapped.index_of(&"A").unwrap();
    let target_idx = mapped.index_of(&"E").unwrap();

    let (path_idx, cost) = mapped
        .graph()
        .bmssp(start_idx, |node| node == target_idx)
        .unwrap();

    let path_nodes: Vec<&str> = path_idx
        .iter()
        .map(|&idx| *mapped.node(idx).unwrap())
        .collect();

    assert_eq!(path_nodes, vec!["A", "B", "D", "E"]);
    assert_eq!(cost, 11);

    println!("Shortest path from A to E: {path_nodes:?} (cost {cost})");

    let parents = mapped.graph().bmssp_all(start_idx);
    let cost_all = parents[target_idx].unwrap().1;
    assert_eq!(cost_all, 11);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bmssp_example() {
        main();
    }
}
