use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use pathfinding_indexed::IndexedGraph;

const SIZE: usize = 64;
const SIDE: usize = SIZE + 1;
const NODE_COUNT: usize = SIDE * SIDE;
const START: usize = index(0, 0);
const GOAL: usize = index(SIZE, SIZE);

const fn index(x: usize, y: usize) -> usize {
    y * SIDE + x
}

fn build_grid_graph() -> IndexedGraph<usize> {
    let mut adjacency = vec![Vec::new(); NODE_COUNT];
    for y in 0..SIDE {
        for x in 0..SIDE {
            let idx = index(x, y);
            if x > 0 {
                adjacency[idx].push((index(x - 1, y), 1));
            }
            if x < SIZE {
                adjacency[idx].push((index(x + 1, y), 1));
            }
            if y > 0 {
                adjacency[idx].push((index(x, y - 1), 1));
            }
            if y < SIZE {
                adjacency[idx].push((index(x, y + 1), 1));
            }
        }
    }
    IndexedGraph::from_adjacency(adjacency)
}

fn build_empty_graph() -> IndexedGraph<usize> {
    IndexedGraph::from_adjacency(vec![Vec::new(); NODE_COUNT])
}

const fn heuristic(node: usize) -> usize {
    let x = node % SIDE;
    let y = node / SIDE;
    let dx = SIZE - x;
    let dy = SIZE - y;
    dx + dy
}

#[library_benchmark]
fn corner_to_corner_astar() {
    let graph = build_grid_graph();
    assert_ne!(graph.astar(START, heuristic, |n| n == GOAL), None);
}

#[library_benchmark]
fn corner_to_corner_bfs() {
    let graph = build_grid_graph();
    assert_ne!(graph.bfs(START, |n| n == GOAL), None);
}

#[library_benchmark]
fn corner_to_corner_bfs_bidirectional() {
    let graph = build_grid_graph();
    assert_ne!(graph.bfs_bidirectional(START, GOAL), None);
}

#[library_benchmark]
fn corner_to_corner_dfs() {
    let graph = build_grid_graph();
    assert_ne!(graph.dfs(START, |n| n == GOAL), None);
}

#[library_benchmark]
fn corner_to_corner_dijkstra() {
    let graph = build_grid_graph();
    assert_ne!(graph.dijkstra(START, |n| n == GOAL), None);
}

#[library_benchmark]
fn corner_to_corner_fringe() {
    let graph = build_grid_graph();
    assert_ne!(graph.fringe(START, heuristic, |n| n == GOAL), None);
}

#[library_benchmark]
fn corner_to_corner_idastar() {
    let graph = build_grid_graph();
    assert_ne!(graph.idastar(START, heuristic, |n| n == GOAL), None);
}

#[library_benchmark]
fn corner_to_corner_iddfs() {
    let graph = build_grid_graph();
    assert_ne!(graph.iddfs(START, |n| n == GOAL), None);
}

#[library_benchmark]
fn no_path_astar() {
    let graph = build_grid_graph();
    assert_eq!(graph.astar(START, |_| 1, |_| false), None);
}

#[library_benchmark]
fn no_path_bfs() {
    let graph = build_grid_graph();
    assert_eq!(graph.bfs(START, |_| false), None);
}

#[library_benchmark]
fn no_path_bfs_bidirectional() {
    let graph = build_empty_graph();
    assert_eq!(graph.bfs_bidirectional(START, GOAL), None);
}

#[library_benchmark]
fn no_path_dfs() {
    let graph = build_grid_graph();
    assert_eq!(graph.dfs(START, |_| false), None);
}

#[library_benchmark]
fn no_path_dijkstra() {
    let graph = build_grid_graph();
    assert_eq!(graph.dijkstra(START, |_| false), None);
}

#[library_benchmark]
fn no_path_fringe() {
    let graph = build_grid_graph();
    assert_eq!(graph.fringe(START, |_| 1, |_| false), None);
}

library_benchmark_group!(
    name = corner_to_corner;
    benchmarks =
        corner_to_corner_astar,
        corner_to_corner_bfs,
        corner_to_corner_bfs_bidirectional,
        corner_to_corner_dfs,
        corner_to_corner_dijkstra,
        corner_to_corner_fringe,
        corner_to_corner_idastar,
        corner_to_corner_iddfs
);

library_benchmark_group!(
    name = no_path;
    benchmarks =
        no_path_astar,
        no_path_bfs,
        no_path_bfs_bidirectional,
        no_path_dfs,
        no_path_dijkstra,
        no_path_fringe
);

main!(library_benchmark_groups = corner_to_corner, no_path);
