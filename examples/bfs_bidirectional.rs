//! This example demonstrates the BFS bidirectional algorithm,
//! and compares it with the regular BFS algorithm.

use pathfinding_indexed::IndexedGraph;
use std::time::Instant;

const SIZE: usize = 64;
const SIDE: usize = SIZE + 1;
const NODE_COUNT: usize = SIDE * SIDE;
const BOTTOM_LEFT: usize = index(0, 0);
const TOP_RIGHT: usize = index(SIZE, SIZE);
const CENTER: usize = index(SIZE / 2, SIZE / 2);

const fn index(x: usize, y: usize) -> usize {
    y * SIDE + x
}

fn build_grid_graph() -> IndexedGraph<u8> {
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

fn main() {
    let graph = build_grid_graph();
    run_corner_to_corner(&graph);
    run_center_to_corner(&graph);
}

/// Corner to corner:
/// =================
///
/// In this case both algorithms will perform similarly.
/// In fact, regular BFS will perform slightly better, since the algorithm is slightly simpler.
///
/// We can understand this in terms of the number of points that need to be searched in order to reach
/// the goal. In the below diagrams this corresponds to the area covered in the final snapshot.
///
/// In both cases every point gets searched - the entire area is filled. For this reason we can intuitively see that
/// regular BFS and bidirectional BFS will perform similarly.
///
/// Regular BFS:
/// ============
///
/// $---------$         $---------$         $---------$         $---------$               $---------$         $---------$
/// |        G|         |        G|         |        G|         |        G|               |FFFFFFF G|         |FFFFFFFFG|
/// |         |         |         |         |         |         |         |               |FFFFFFFF |         |FFFFFFFFF|
/// |         |         |         |         |         |         |         |               |FFFFFFFFF|         |FFFFFFFFF|
/// |         |         |         |         |         |         |         |               |FFFFFFFFF|         |FFFFFFFFF|
/// |         |    =>   |         |    =>   |         |    =>   |         |   => ... =>   |FFFFFFFFF|    =>   |FFFFFFFFF|
/// |         |         |         |         |         |         |F        |               |FFFFFFFFF|         |FFFFFFFFF|
/// |         |         |         |         |F        |         |FF       |               |FFFFFFFFF|         |FFFFFFFFF|
/// |         |         |F        |         |FF       |         |FFF      |               |FFFFFFFFF|         |FFFFFFFFF|
/// |S        |         |SF       |         |SFF      |         |SFFF     |               |SFFFFFFFF|         |SFFFFFFFF|
/// $---------$         $---------$         $---------$         $---------$               $---------$         $---------$
///
/// Bidirectional BFS:
/// ==================
///
/// $---------$         $---------$         $---------$         $---------$               $---------$         $---------$
/// |        G|         |       BG|         |      BBG|         |     BBBG|               | BBBBBBBG|         |FBBBBBBBG|
/// |         |         |        B|         |       BB|         |      BBB|               |F BBBBBBB|         |FFBBBBBBB|
/// |         |         |         |         |        B|         |       BB|               |FF BBBBBB|         |FFFBBBBBB|
/// |         |         |         |         |         |         |        B|               |FFF BBBBB|         |FFFFBBBBB|
/// |         |    =>   |         |    =>   |         |    =>   |         |   => ... =>   |FFFF BBBB|    =>   |FFFFFBBBB|
/// |         |         |         |         |         |         |F        |               |FFFFF BBB|         |FFFFFFBBB|
/// |         |         |         |         |F        |         |FF       |               |FFFFFF BB|         |FFFFFFFBB|
/// |         |         |F        |         |FF       |         |FFF      |               |FFFFFFF B|         |FFFFFFFFB|
/// |S        |         |SF       |         |SFF      |         |SFFF     |               |SFFFFFFF |         |SFFFFFFFF|
/// $---------$         $---------$         $---------$         $---------$               $---------$         $---------$
fn run_corner_to_corner(graph: &IndexedGraph<u8>) {
    let instant = Instant::now();
    graph.bfs(BOTTOM_LEFT, |node| node == TOP_RIGHT);
    let duration_bfs = instant.elapsed();

    let instant = Instant::now();
    let _ = graph.bfs_bidirectional(BOTTOM_LEFT, TOP_RIGHT);
    let duration_bfs_bidirectional = instant.elapsed();

    print!(
        "
Corner to Corner
================
BFS took {duration_bfs:?}
Bidirectional BFS took {duration_bfs_bidirectional:?}
"
    );
}

/// Center to corner:
/// =================
///
/// In this case bidirectional BFS will outperform regular BFS.
///
/// We can understand this in terms of the number of points that need to be searched in order to reach
/// the goal. In the below diagrams this corresponds to the area covered in the final snapshot.
///
/// In this case for the regular BFS every point still needs to be searched - again, the entire area is filled.
/// However, for the bidirectional BFS some points remain unsearched - the entire area is not filled. For this
/// reason we can intuitively see that bidirectional BFS will outperform regular BFS here.
///
/// Regular BFS:
/// ============
///
/// $---------$         $---------$         $---------$               $---------$          $---------$
/// |        G|         |        G|         |        G|               | FFFFFFFG|          |FFFFFFFFG|
/// |         |         |         |         |         |               |FFFFFFFFF|          |FFFFFFFFF|
/// |         |         |    F    |         |    F    |               |FFFFFFFFF|          |FFFFFFFFF|
/// |         |         |   FFF   |         |   FFF   |               |FFFFFFFFF|          |FFFFFFFFF|
/// |    S    |    =>   |  FFSFF  |    =>   |  FFSFF  |   => ... =>   |FFFFSFFFF|    =>    |FFFFSFFFF|
/// |         |         |   FFF   |         |   FFF   |               |FFFFFFFFF|          |FFFFFFFFF|
/// |         |         |    F    |         |    F    |               |FFFFFFFFF|          |FFFFFFFFF|
/// |         |         |         |         |         |               |FFFFFFFFF|          |FFFFFFFFF|
/// |         |         |         |         |         |               | FFFFFFF |          |FFFFFFFFF|
/// $---------$         $---------$         $---------$               $---------$          $---------$
///
/// Bidirectional BFS:
/// ==================
///
/// $---------$         $---------$         $---------$         $---------$         $---------$
/// |        G|         |       BG|         |      BBG|         |     BBBG|         |    FBBBG|
/// |         |         |        B|         |       BB|         |    F BBB|         |   FFFBBB|
/// |         |         |         |         |    F    |         |   FFF  B|         |  FFFFFBB|
/// |         |         |    F    |         |   FFF   |         |  FFFFF  |         | FFFFFFFB|
/// |    S    |    =>   |   FSF   |    =>   |  FFSFF  |    =>   | FFFSFFF |    =>   |FFFFSFFFF|
/// |         |         |    F    |         |   FFF   |         |  FFFFF  |         | FFFFFFF |
/// |         |         |         |         |    F    |         |   FFF   |         |  FFFFF  |
/// |         |         |         |         |         |         |    F    |         |   FFF   |
/// |         |         |         |         |         |         |         |         |    F    |
/// $---------$         $---------$         $---------$         $---------$         $---------$
fn run_center_to_corner(graph: &IndexedGraph<u8>) {
    let instant = Instant::now();
    graph.bfs(CENTER, |node| node == TOP_RIGHT);
    let duration_bfs = instant.elapsed();

    let instant = Instant::now();
    let _ = graph.bfs_bidirectional(CENTER, TOP_RIGHT);
    let duration_bfs_bidirectional = instant.elapsed();

    print!(
        "
Center to Corner
================
BFS took {duration_bfs:?}
Bidirectional BFS took {duration_bfs_bidirectional:?}
"
    );
}
