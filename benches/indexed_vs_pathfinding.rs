use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use pathfinding::directed::{astar as pf_astar, dijkstra as pf_dijkstra};
use pathfinding_indexed::IndexedGraph;

const SIZE: usize = 64;
const SIDE: usize = SIZE + 1;
const NODE_COUNT: usize = SIDE * SIDE;
const START: usize = index(0, 0);
const GOAL: usize = index(SIZE, SIZE);

const fn index(x: usize, y: usize) -> usize {
    y * SIDE + x
}

fn build_grid_adjacency() -> Vec<Vec<(usize, usize)>> {
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
    adjacency
}

const fn heuristic(node: &usize) -> usize {
    let x = *node % SIDE;
    let y = *node / SIDE;
    let dx = SIZE - x;
    let dy = SIZE - y;
    dx + dy
}

fn indexed_corner_to_corner_dijkstra(c: &mut Criterion) {
    let adjacency = build_grid_adjacency();
    let graph = IndexedGraph::from_adjacency(adjacency);
    c.bench_function("indexed_corner_to_corner_dijkstra", |b| {
        b.iter(|| assert_ne!(graph.dijkstra(START, |n| n == GOAL), None));
    });
}

fn pathfinding_corner_to_corner_dijkstra(c: &mut Criterion) {
    let adjacency = build_grid_adjacency();
    c.bench_function("pathfinding_corner_to_corner_dijkstra", |b| {
        b.iter(|| {
            let result = pf_dijkstra::dijkstra(
                &START,
                |&node| adjacency[node].iter().copied(),
                |&node| node == GOAL,
            );
            assert_ne!(result, None);
        });
    });
}

fn indexed_corner_to_corner_astar(c: &mut Criterion) {
    let adjacency = build_grid_adjacency();
    let graph = IndexedGraph::from_adjacency(adjacency);
    c.bench_function("indexed_corner_to_corner_astar", |b| {
        b.iter(|| assert_ne!(graph.astar(START, |n| heuristic(&n), |n| n == GOAL), None));
    });
}

fn pathfinding_corner_to_corner_astar(c: &mut Criterion) {
    let adjacency = build_grid_adjacency();
    c.bench_function("pathfinding_corner_to_corner_astar", |b| {
        b.iter(|| {
            let result = pf_astar::astar(
                &START,
                |&node| adjacency[node].iter().copied(),
                heuristic,
                |&node| node == GOAL,
            );
            assert_ne!(result, None);
        });
    });
}

criterion_group!(
    benches,
    indexed_corner_to_corner_dijkstra,
    pathfinding_corner_to_corner_dijkstra,
    indexed_corner_to_corner_astar,
    pathfinding_corner_to_corner_astar,
);
criterion_main!(benches);
