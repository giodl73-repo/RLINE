use rgraph_core::{shortest_connector_path, undirected_boundary_metrics};
use ropt_core::{exact_budgeted_selection, BudgetItem};
use rstat_core::scoring::{exponential_recency_score, weighted_component_score};

#[test]
fn l1_missing_bridge_fixture_recovers_connector_top_k_misses() {
    let adjacency = vec![vec![1_usize, 3], vec![0, 2], vec![1], vec![0]];
    let relevance: [f64; 4] = [1.0, 0.2, 0.95, 0.9];

    let mut top_k: Vec<usize> = (1..relevance.len()).collect();
    top_k.sort_by(|&left, &right| {
        relevance[right]
            .total_cmp(&relevance[left])
            .then_with(|| left.cmp(&right))
    });

    assert_eq!(&top_k[..2], &[2, 3]);
    assert!(!top_k[..2].contains(&1));

    let connector = shortest_connector_path(&adjacency, &[0], &[2])
        .unwrap()
        .expect("target should be connected through bridge");
    assert_eq!(connector.nodes, vec![0, 1, 2]);
    assert_eq!(connector.bridge_nodes, vec![1]);
}

#[test]
fn l1_noisy_periphery_fixture_has_worse_boundary_than_bridge_crop() {
    let adjacency = vec![vec![1_usize, 3], vec![0, 2], vec![1], vec![0, 4], vec![3]];
    let bridge_crop = vec![true, true, true, false, false];
    let noisy_crop = vec![true, false, true, true, false];

    let bridge_metrics = undirected_boundary_metrics(&adjacency, &bridge_crop).unwrap();
    let noisy_metrics = undirected_boundary_metrics(&adjacency, &noisy_crop).unwrap();

    assert!(bridge_metrics.boundary_edges < noisy_metrics.boundary_edges);
    assert!(bridge_metrics.conductance < noisy_metrics.conductance);
}

#[test]
fn l1_stale_duplicate_fixture_ranks_current_source_above_old_duplicate() {
    let relevance = 0.9;
    let current_recency = exponential_recency_score(1.0, 10.0).unwrap();
    let stale_recency = exponential_recency_score(40.0, 10.0).unwrap();

    let current_score = weighted_component_score(&[relevance, current_recency], &[0.7, 0.3])
        .expect("current source score should compute");
    let stale_score = weighted_component_score(&[relevance, stale_recency], &[0.7, 0.3])
        .expect("stale source score should compute");

    assert!(current_score > stale_score);
}

#[test]
fn l1_token_budget_fixture_beats_naive_score_order() {
    let items = [
        BudgetItem {
            score: 9.0,
            cost: 9,
        },
        BudgetItem {
            score: 5.0,
            cost: 5,
        },
        BudgetItem {
            score: 5.0,
            cost: 5,
        },
    ];

    let naive = BudgetItem {
        score: items[0].score,
        cost: items[0].cost,
    };
    let exact = exact_budgeted_selection(&items, 10).unwrap();

    assert_eq!(exact.selected_indices, vec![1, 2]);
    assert!(exact.total_score > naive.score);
    assert_eq!(exact.total_cost, 10);
}
