/// Example demonstrating how to use an adjacency list graph representation.
/// This example shows a weighted directed graph and uses Dijkstra's algorithm to find
/// the shortest path.
use pathfinding_indexed::IndexedGraphMap;
use std::collections::HashMap;

fn main() {
    // Create a weighted graph using adjacency list.
    // Each node maps to a list of (neighbor, weight) pairs.
    let graph: HashMap<&str, Vec<(&str, u32)>> = [
        ("A", vec![("B", 4), ("C", 2)]),
        ("B", vec![("C", 1), ("D", 5)]),
        ("C", vec![("D", 8), ("E", 10)]),
        ("D", vec![("E", 2)]),
        ("E", vec![]),
    ]
    .into_iter()
    .collect();

    let mapped = IndexedGraphMap::from_nodes_and_successors(graph.keys().copied(), |node| {
        graph.get(node).cloned().unwrap_or_default()
    });

    let start = mapped.index_of(&"A").unwrap();
    let goal = mapped.index_of(&"E").unwrap();

    let result = mapped.graph().dijkstra(start, |node| node == goal);

    match result {
        Some((path, cost)) => {
            let path_nodes: Vec<&str> =
                path.iter().map(|&idx| *mapped.node(idx).unwrap()).collect();
            println!("Shortest path from A to E:");
            println!("  Path: {path_nodes:?}");
            println!("  Total cost: {cost}");
            // The shortest path is A -> B (4) -> D (5) -> E (2) = 11
            assert_eq!(path_nodes, vec!["A", "B", "D", "E"]);
            assert_eq!(cost, 11);
        }
        None => println!("No path found"),
    }

    // Find another path: A to D
    let goal = mapped.index_of(&"D").unwrap();
    let result2 = mapped.graph().dijkstra(start, |node| node == goal);

    match result2 {
        Some((path, cost)) => {
            let path_nodes: Vec<&str> =
                path.iter().map(|&idx| *mapped.node(idx).unwrap()).collect();
            println!("\nShortest path from A to D:");
            println!("  Path: {path_nodes:?}");
            println!("  Total cost: {cost}");
        }
        None => println!("No path found"),
    }

    println!("\nExample completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacency_list_example() {
        main();
    }
}
