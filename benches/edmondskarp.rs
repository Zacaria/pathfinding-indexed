use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use pathfinding_faster::IndexedGraph;
use std::collections::HashMap;

/// Return a list of edges with their capacities.
fn successors_wikipedia() -> Vec<((char, char), i32)> {
    vec![
        ("AB", 3),
        ("AD", 3),
        ("BC", 4),
        ("CA", 3),
        ("CD", 1),
        ("CE", 2),
        ("DE", 2),
        ("DF", 6),
        ("EB", 1),
        ("EG", 1),
        ("FG", 9),
    ]
    .into_iter()
    .map(|(s, c)| {
        let mut name = s.chars();
        ((name.next().unwrap(), name.next().unwrap()), c)
    })
    .collect()
}

const NODES: [char; 8] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];

const fn to_index(c: char) -> usize {
    match c {
        'A' => 0,
        'B' => 1,
        'C' => 2,
        'D' => 3,
        'E' => 4,
        'F' => 5,
        'G' => 6,
        'H' => 7,
        _ => usize::MAX,
    }
}

const fn to_char(idx: usize) -> char {
    NODES[idx]
}

fn build_graph() -> IndexedGraph<i32> {
    let mut adjacency = vec![Vec::new(); 8];
    for ((from, to), cap) in successors_wikipedia() {
        adjacency[to_index(from)].push((to_index(to), cap));
    }
    IndexedGraph::from_adjacency(adjacency)
}

type FlowResult = (Vec<((usize, usize), i32)>, i32, Vec<((usize, usize), i32)>);

fn check_wikipedia_result(flows: FlowResult) {
    let (caps, total, _cuts) = flows;
    assert_eq!(caps.len(), 8);
    let caps = caps
        .into_iter()
        .map(|((from, to), cap)| ((to_char(from), to_char(to)), cap))
        .collect::<HashMap<(char, char), i32>>();
    assert_eq!(caps[&('A', 'B')], 2);
    assert_eq!(caps[&('A', 'D')], 3);
    assert_eq!(caps[&('B', 'C')], 2);
    assert_eq!(caps[&('C', 'D')], 1);
    assert_eq!(caps[&('C', 'E')], 1);
    assert_eq!(caps[&('D', 'F')], 4);
    assert_eq!(caps[&('E', 'G')], 1);
    assert_eq!(caps[&('F', 'G')], 4);
    assert_eq!(total, 5);
}

fn wikipedia_example(c: &mut Criterion) {
    let graph = build_graph();
    c.bench_function("wikipedia_example", |b| {
        b.iter(|| check_wikipedia_result(graph.edmonds_karp(to_index('A'), to_index('G'))));
    });
}

criterion_group!(benches, wikipedia_example);
criterion_main!(benches);
