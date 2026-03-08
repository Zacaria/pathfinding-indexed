/// Example demonstrating how to use BFS on an unweighted graph.
/// In unweighted graphs, edges all have the same cost.
use pathfinding_indexed::IndexedGraphMap;
use std::collections::HashMap;

fn main() {
    // Define an unweighted graph as an adjacency list.
    let graph: HashMap<&str, Vec<&str>> = [
        ("A", vec!["B", "C"]),
        ("B", vec!["A", "D", "E"]),
        ("C", vec!["A", "F"]),
        ("D", vec!["B"]),
        ("E", vec!["B", "F"]),
        ("F", vec!["C", "E"]),
    ]
    .into_iter()
    .collect();

    let mapped = IndexedGraphMap::from_nodes_and_successors(graph.keys().copied(), |node| {
        graph
            .get(node)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|neighbor| (neighbor, 1u8))
            .collect::<Vec<_>>()
    });

    let start = mapped.index_of(&"A").unwrap();
    let goal = mapped.index_of(&"F").unwrap();

    let result = mapped.graph().bfs(start, |node| node == goal);

    match result {
        Some(path) => {
            let path_nodes: Vec<&str> =
                path.iter().map(|&idx| *mapped.node(idx).unwrap()).collect();
            println!("Shortest path from A to F: {path_nodes:?}");
            println!("Number of hops: {}", path_nodes.len() - 1);
            assert_eq!(path_nodes, vec!["A", "C", "F"]);
            assert_eq!(path_nodes.len() - 1, 2); // 2 hops
        }
        None => println!("No path found"),
    }

    // Example 2: Find path from A to E
    let goal = mapped.index_of(&"E").unwrap();
    let result2 = mapped.graph().bfs(start, |node| node == goal);

    match result2 {
        Some(path) => {
            let path_nodes: Vec<&str> =
                path.iter().map(|&idx| *mapped.node(idx).unwrap()).collect();
            println!("\nShortest path from A to E: {path_nodes:?}");
            println!("Number of hops: {}", path_nodes.len() - 1);
        }
        None => println!("No path found"),
    }

    println!("\nExample completed successfully!");
    println!("\nThis demonstrates BFS on an unweighted graph where:");
    println!("- All edges have equal cost (1 hop)");
    println!("- The graph is indexed and stored explicitly");
    println!("- BFS finds the path with the fewest hops");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unweighted_bfs_example() {
        main();
    }
}
