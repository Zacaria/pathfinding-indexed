use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use pathfinding_indexed::IndexedUndirectedGraph;

fn build_component_graph() -> IndexedUndirectedGraph<u8> {
    let component_size = 100;
    let components = 100;
    let node_count = component_size * components;
    let mut edges = Vec::new();
    for c in 0..components {
        let base = c * component_size;
        for i in 0..(component_size - 1) {
            let u = base + i;
            let v = base + i + 1;
            edges.push((u, v, 1));
        }
    }
    IndexedUndirectedGraph::from_edges(node_count, edges)
}

fn bench_separate_components(c: &mut Criterion) {
    let graph = build_component_graph();
    c.bench_function("separate_components", |b| {
        b.iter(|| {
            let (mapping, _) = graph.separate_components();
            assert_eq!(mapping.len(), graph.node_count());
        });
    });
}

criterion_group!(benches, bench_separate_components);
criterion_main!(benches);
