/// Example demonstrating how to encapsulate graph logic in a struct.
/// This example shows a road network with both Dijkstra and A* algorithms.
use pathfinding_faster::IndexedGraphMap;
use std::collections::HashMap;

struct RoadNetwork {
    // adjacency list: city name -> list of (neighbor, distance)
    connections: HashMap<String, Vec<(String, u32)>>,
    // coordinates for heuristic (optional)
    coordinates: HashMap<String, (i32, i32)>,
}

impl RoadNetwork {
    fn new() -> Self {
        Self {
            connections: HashMap::new(),
            coordinates: HashMap::new(),
        }
    }

    fn add_road(&mut self, from: &str, to: &str, distance: u32) {
        self.connections
            .entry(from.to_string())
            .or_default()
            .push((to.to_string(), distance));
    }

    fn add_bidirectional_road(&mut self, city1: &str, city2: &str, distance: u32) {
        self.add_road(city1, city2, distance);
        self.add_road(city2, city1, distance);
    }

    fn add_coordinates(&mut self, city: &str, x: i32, y: i32) {
        self.coordinates.insert(city.to_string(), (x, y));
    }

    fn successors(&self, city: &str) -> Vec<(String, u32)> {
        self.connections.get(city).cloned().unwrap_or_default()
    }

    fn to_indexed(&self) -> IndexedGraphMap<String, u32> {
        let nodes = self.connections.keys().cloned().collect::<Vec<_>>();
        IndexedGraphMap::from_nodes_and_successors(nodes, |city| self.successors(city))
    }

    fn find_path_dijkstra(&self, start: &str, goal: &str) -> Option<(Vec<String>, u32)> {
        let mapped = self.to_indexed();
        let start_idx = mapped.index_of(&start.to_string())?;
        let goal_idx = mapped.index_of(&goal.to_string())?;
        let (path, cost) = mapped
            .graph()
            .dijkstra(start_idx, |node| node == goal_idx)?;
        let path_nodes = path
            .iter()
            .map(|&idx| mapped.node(idx).unwrap().clone())
            .collect();
        Some((path_nodes, cost))
    }

    fn find_path_astar(&self, start: &str, goal: &str) -> Option<(Vec<String>, u32)> {
        let mapped = self.to_indexed();
        let start_idx = mapped.index_of(&start.to_string())?;
        let goal_idx = mapped.index_of(&goal.to_string())?;
        let coords = mapped
            .nodes()
            .iter()
            .map(|name| self.coordinates.get(name).copied().unwrap_or((0, 0)))
            .collect::<Vec<_>>();
        let (goal_x, goal_y) = coords[goal_idx];
        let heuristic = |node: usize| -> u32 {
            let (x, y) = coords[node];
            let dx = goal_x.abs_diff(x);
            let dy = goal_y.abs_diff(y);
            dx + dy
        };
        let (path, cost) = mapped
            .graph()
            .astar(start_idx, heuristic, |node| node == goal_idx)?;
        let path_nodes = path
            .iter()
            .map(|&idx| mapped.node(idx).unwrap().clone())
            .collect();
        Some((path_nodes, cost))
    }
}

fn main() {
    let mut network = RoadNetwork::new();

    // Build a road network
    network.add_bidirectional_road("CityA", "CityB", 10);
    network.add_bidirectional_road("CityA", "CityC", 15);
    network.add_bidirectional_road("CityB", "CityD", 12);
    network.add_bidirectional_road("CityC", "CityD", 10);
    network.add_bidirectional_road("CityB", "CityE", 8);
    network.add_bidirectional_road("CityD", "CityE", 5);

    // Add coordinates for A* heuristic
    network.add_coordinates("CityA", 0, 0);
    network.add_coordinates("CityB", 10, 0);
    network.add_coordinates("CityC", 0, 15);
    network.add_coordinates("CityD", 10, 12);
    network.add_coordinates("CityE", 15, 8);

    println!("Road Network Pathfinding Example\n");

    // Find path using Dijkstra
    if let Some((path, cost)) = network.find_path_dijkstra("CityA", "CityE") {
        println!("Dijkstra's Algorithm:");
        println!("  Path from CityA to CityE: {path:?}");
        println!("  Total distance: {cost}");
    }

    // Find path using A*
    if let Some((path, cost)) = network.find_path_astar("CityA", "CityE") {
        println!("\nA* Algorithm:");
        println!("  Path from CityA to CityE: {path:?}");
        println!("  Total distance: {cost}");
    }

    // Another example
    if let Some((path, cost)) = network.find_path_dijkstra("CityA", "CityD") {
        println!("\nDijkstra from CityA to CityD:");
        println!("  Path: {path:?}");
        println!("  Total distance: {cost}");
        assert_eq!(cost, 22); // A -> B (10) -> D (12) = 22
    }

    println!("\nExample completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_struct_example() {
        main();
    }
}
