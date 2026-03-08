use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use pathfinding_indexed::IndexedGraph;

const NODE_COUNT: usize = 4096;
const OUT_DEGREE: usize = 4;
const START: usize = 0;
const GOAL: usize = NODE_COUNT - 1;

const fn next_u32(state: &mut u64) -> u32 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    (*state >> 32) as u32
}

fn build_constant_degree_graph() -> IndexedGraph<usize> {
    let mut adjacency = (0..NODE_COUNT)
        .map(|_| Vec::with_capacity(OUT_DEGREE))
        .collect::<Vec<_>>();
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    for (node, edges) in adjacency.iter_mut().enumerate() {
        let next = (node + 1) % NODE_COUNT;
        edges.push((next, 1));
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

fn constant_degree_dijkstra_all(c: &mut Criterion) {
    let graph = build_constant_degree_graph();
    c.bench_function("constant_degree_dijkstra_all", |b| {
        b.iter(|| {
            let out = graph.dijkstra_all(START);
            assert!(out[GOAL].is_some());
        });
    });
}

fn constant_degree_bmssp_all(c: &mut Criterion) {
    let graph = build_constant_degree_graph();
    c.bench_function("constant_degree_bmssp_all", |b| {
        b.iter(|| {
            let out = graph.bmssp_all(START);
            assert!(out[GOAL].is_some());
        });
    });
}

criterion_group!(
    benches,
    constant_degree_dijkstra_all,
    constant_degree_bmssp_all
);
criterion_main!(benches);
