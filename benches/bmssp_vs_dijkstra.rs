use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use pathfinding_indexed::IndexedGraph;

const NODE_COUNT: usize = 4096;
const OUT_DEGREE: usize = 32;
const START: usize = 0;
const GOAL: usize = NODE_COUNT - 1;

const fn next_u32(state: &mut u64) -> u32 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    (*state >> 32) as u32
}

fn build_dense_graph() -> IndexedGraph<usize> {
    let mut adjacency = (0..NODE_COUNT)
        .map(|_| Vec::with_capacity(OUT_DEGREE))
        .collect::<Vec<_>>();
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    for (node, edges) in adjacency.iter_mut().enumerate() {
        if node + 1 < NODE_COUNT {
            edges.push((node + 1, 1));
        }
        while edges.len() < OUT_DEGREE {
            let target = (next_u32(&mut state) as usize) % NODE_COUNT;
            if target == node {
                continue;
            }
            let weight = (next_u32(&mut state) as usize % 9) + 1;
            edges.push((target, weight));
        }
    }
    IndexedGraph::from_adjacency(adjacency)
}

fn dense_graph_dijkstra(c: &mut Criterion) {
    let graph = build_dense_graph();
    c.bench_function("dense_graph_dijkstra", |b| {
        b.iter(|| assert_ne!(graph.dijkstra(START, |n| n == GOAL), None));
    });
}

fn dense_graph_bmssp(c: &mut Criterion) {
    let graph = build_dense_graph();
    c.bench_function("dense_graph_bmssp", |b| {
        b.iter(|| assert_ne!(graph.bmssp(START, |n| n == GOAL), None));
    });
}

criterion_group!(benches, dense_graph_dijkstra, dense_graph_bmssp);
criterion_main!(benches);
