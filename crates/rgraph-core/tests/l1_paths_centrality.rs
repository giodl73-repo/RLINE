use rgraph_core::{
    articulation_points, articulation_points_with_filter, assignment_labels_connected, bridges,
    bridges_with_filter, connected_components, connected_components_in_nodes_with_filter,
    edge_betweenness, node_subset_connected, reachable_nodes_with_filter, shortest_path_distance,
    shortest_path_distance_with_filter, single_source_shortest_paths, undirected_edge_cut, Bridge,
    DirectedWeightedGraph, GraphError, WeightedEdge,
};

#[derive(Debug, Clone)]
struct TestGraph {
    edges: Vec<Vec<WeightedEdge<u32>>>,
}

impl TestGraph {
    fn new(node_count: usize) -> Self {
        Self {
            edges: vec![Vec::new(); node_count],
        }
    }

    fn edge(mut self, id: u32, from: usize, to: usize, weight: f64) -> Self {
        self.edges[from].push(WeightedEdge {
            id,
            target: to,
            weight,
        });
        self
    }
}

impl DirectedWeightedGraph for TestGraph {
    type EdgeId = u32;

    fn node_count(&self) -> usize {
        self.edges.len()
    }

    fn outgoing_edges(&self, source: usize) -> Vec<WeightedEdge<Self::EdgeId>> {
        self.edges[source].clone()
    }
}

#[test]
fn l1_weighted_shortest_paths_prefer_lower_cost_indirect_route() {
    let graph = TestGraph::new(4)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 3, 1.0)
        .edge(3, 0, 2, 0.5)
        .edge(4, 2, 3, 10.0)
        .edge(5, 0, 3, 5.0);

    let tree = single_source_shortest_paths(&graph, 0).unwrap();

    assert_eq!(shortest_path_distance(&graph, 0, 3).unwrap(), Some(2.0));
    assert_eq!(tree.distance_to(3), Some(2.0));
    assert_eq!(tree.predecessors[3][0].edge_id, 2);
}

#[test]
fn l1_filter_changes_path_and_reachability() {
    let graph = TestGraph::new(5)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 4, 1.0)
        .edge(3, 0, 2, 1.0)
        .edge(4, 2, 3, 1.0)
        .edge(5, 3, 4, 1.0);

    assert_eq!(shortest_path_distance(&graph, 0, 4).unwrap(), Some(2.0));
    assert_eq!(
        shortest_path_distance_with_filter(&graph, 0, 4, |edge| edge != 2).unwrap(),
        Some(3.0)
    );
    assert_eq!(
        reachable_nodes_with_filter(&graph, 0, |edge| !matches!(edge, 2 | 5)).unwrap(),
        vec![0, 1, 2, 3]
    );
}

#[test]
fn l1_centrality_identifies_bridge_edge() {
    let graph = TestGraph::new(5)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 2, 1.0)
        .edge(3, 2, 3, 1.0)
        .edge(4, 3, 4, 1.0)
        .edge(5, 0, 4, 10.0);

    let centrality = edge_betweenness(&graph).unwrap();

    assert!(centrality[&2] >= centrality[&1]);
    assert!(centrality[&3] >= centrality[&4]);
    assert!(centrality[&2] > centrality.get(&5).copied().unwrap_or(0.0));
}

#[test]
fn l1_parallel_shortest_path_overflow_is_rejected_before_centrality() {
    let mut graph = TestGraph::new(1100);
    let mut edge_id = 0u32;
    for source in 0..1099 {
        graph = graph.edge(edge_id, source, source + 1, 1.0);
        edge_id += 1;
        graph = graph.edge(edge_id, source, source + 1, 1.0);
        edge_id += 1;
    }

    match edge_betweenness(&graph) {
        Err(GraphError::NonFinitePathCount { node, count }) => {
            assert!(node > 0);
            assert!(count.is_infinite());
        }
        other => panic!("expected path-count overflow error, got {other:?}"),
    }
}

#[test]
fn l1_shortest_path_distance_overflow_is_rejected() {
    let graph = TestGraph::new(3)
        .edge(1, 0, 1, f64::MAX)
        .edge(2, 1, 2, f64::MAX);

    match shortest_path_distance(&graph, 0, 2) {
        Err(GraphError::NonFiniteDistance {
            from,
            target,
            distance,
        }) => {
            assert_eq!((from, target), (1, 2));
            assert!(distance.is_infinite());
        }
        other => panic!("expected distance overflow error, got {other:?}"),
    }
}

#[test]
fn l1_invalid_target_from_adapter_is_reported() {
    let graph = TestGraph::new(2).edge(9, 0, 3, 1.0);

    let err = shortest_path_distance(&graph, 0, 1).unwrap_err();

    assert_eq!(
        err,
        GraphError::NodeOutOfBounds {
            node: 3,
            node_count: 2
        }
    );
}

#[test]
fn l1_connected_components_support_subset_and_filter() {
    let graph = TestGraph::new(7)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 2, 1.0)
        .edge(3, 3, 4, 1.0)
        .edge(4, 4, 5, 1.0)
        .edge(5, 5, 6, 1.0);

    assert_eq!(
        connected_components(&graph).unwrap(),
        vec![vec![0, 1, 2], vec![3, 4, 5, 6]]
    );
    assert_eq!(
        connected_components_in_nodes_with_filter(&graph, &[0, 1, 2, 4, 5, 6], |edge| edge != 5)
            .unwrap(),
        vec![vec![0, 1, 2], vec![4, 5], vec![6]]
    );
}

#[test]
fn l1_connected_components_are_direction_invariant_for_boundary_adapters() {
    let graph = TestGraph::new(5)
        .edge(1, 1, 0, 1.0)
        .edge(2, 2, 1, 1.0)
        .edge(3, 4, 3, 1.0);

    assert_eq!(
        connected_components(&graph).unwrap(),
        vec![vec![0, 1, 2], vec![3, 4]]
    );
    assert_eq!(
        connected_components_in_nodes_with_filter(&graph, &[0, 1, 2, 3], |_| true).unwrap(),
        vec![vec![0, 1, 2], vec![3]]
    );
}

#[test]
fn l1_bridges_ignore_cycles_and_respect_filters() {
    let graph = TestGraph::new(6)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 2, 1.0)
        .edge(3, 2, 0, 1.0)
        .edge(4, 2, 3, 1.0)
        .edge(5, 3, 4, 1.0)
        .edge(6, 4, 5, 1.0)
        .edge(7, 5, 3, 1.0);

    assert_eq!(
        bridges(&graph).unwrap(),
        vec![Bridge {
            source: 2,
            target: 3,
            edge_id: 4
        }]
    );
    assert_eq!(
        bridges_with_filter(&graph, |edge| edge != 7).unwrap(),
        vec![
            Bridge {
                source: 2,
                target: 3,
                edge_id: 4
            },
            Bridge {
                source: 3,
                target: 4,
                edge_id: 5
            },
            Bridge {
                source: 4,
                target: 5,
                edge_id: 6
            }
        ]
    );
}

#[test]
fn l1_articulation_points_track_cut_vertices_and_filters() {
    let graph = TestGraph::new(7)
        .edge(1, 0, 1, 1.0)
        .edge(2, 1, 2, 1.0)
        .edge(3, 2, 0, 1.0)
        .edge(4, 2, 3, 1.0)
        .edge(5, 3, 4, 1.0)
        .edge(6, 4, 5, 1.0)
        .edge(7, 5, 3, 1.0)
        .edge(8, 3, 6, 1.0);

    assert_eq!(articulation_points(&graph).unwrap(), vec![2, 3]);
    assert_eq!(
        articulation_points_with_filter(&graph, |edge| edge != 7).unwrap(),
        vec![2, 3, 4]
    );
}

#[test]
fn l1_undirected_edge_cut_matches_redistricting_fixture() {
    let adjacency = vec![vec![1_usize, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let assignment = vec![0_u32, 0, 1, 1];

    assert_eq!(undirected_edge_cut(&adjacency, &assignment).unwrap(), 2);
}

#[test]
fn l1_undirected_edge_cut_is_direction_invariant_for_boundary_lists() {
    let symmetric = vec![vec![1_usize], vec![0, 2], vec![1, 3], vec![2]];
    let high_to_low_only = vec![vec![], vec![0_usize], vec![1], vec![2]];
    let assignment = vec![0_usize, 0, 1, 1];

    assert_eq!(undirected_edge_cut(&symmetric, &assignment).unwrap(), 1);
    assert_eq!(
        undirected_edge_cut(&high_to_low_only, &assignment).unwrap(),
        1
    );
}

#[test]
fn l1_assignment_labels_connected_matches_redistricting_fixture() {
    let adjacency = vec![vec![1_usize], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]];
    let connected_assignment = vec![0_usize, 0, 1, 1, 1];
    let disconnected_assignment = vec![0_usize, 1, 0, 1, 1];

    assert!(assignment_labels_connected(&adjacency, &connected_assignment, 0..2).unwrap());
    assert!(!assignment_labels_connected(&adjacency, &disconnected_assignment, 0..2).unwrap());
}

#[test]
fn l1_assignment_label_connectivity_is_direction_invariant() {
    let symmetric = vec![vec![1_usize], vec![0, 2], vec![1, 3], vec![2]];
    let high_to_low_only = vec![vec![], vec![0_usize], vec![1], vec![2]];
    let assignment = vec![0_usize, 0, 0, 0];

    assert!(assignment_labels_connected(&symmetric, &assignment, 0..1).unwrap());
    assert!(assignment_labels_connected(&high_to_low_only, &assignment, 0..1).unwrap());
}

#[test]
fn l1_node_subset_connected_matches_redistricting_fixture() {
    let adjacency = vec![vec![1_usize], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]];

    assert!(node_subset_connected(&adjacency, &[1_usize, 2, 3]).unwrap());
    assert!(!node_subset_connected(&adjacency, &[0_usize, 4]).unwrap());
}

#[test]
fn l1_node_subset_connectivity_is_direction_invariant() {
    let symmetric = vec![vec![1_usize], vec![0, 2], vec![1, 3], vec![2]];
    let high_to_low_only = vec![vec![], vec![0_usize], vec![1], vec![2]];

    assert!(node_subset_connected(&symmetric, &[0_usize, 1, 2, 3]).unwrap());
    assert!(node_subset_connected(&high_to_low_only, &[0_usize, 1, 2, 3]).unwrap());
}
