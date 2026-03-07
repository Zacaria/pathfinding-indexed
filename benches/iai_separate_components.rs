use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use pathfinding_faster::IndexedUndirectedGraph;

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

#[library_benchmark]
fn separate_components() {
    let graph = build_component_graph();
    let (mapping, _) = graph.separate_components();
    assert_eq!(mapping.len(), graph.node_count());
}

library_benchmark_group!(name = separate_components_group; benchmarks = separate_components);

main!(library_benchmark_groups = separate_components_group);
