use ropt_core::{
    crowding_distance, derive_seed, dominates, fast_non_dominated_sort, ObjectiveVector, RoptError,
    SeedPart,
};

#[derive(Debug)]
struct Candidate {
    compactness: f64,
    partisan_deviation: f64,
    vra_deficit: f64,
}

impl ObjectiveVector for Candidate {
    fn objective_count(&self) -> usize {
        3
    }

    fn objective_value(&self, index: usize) -> f64 {
        match index {
            0 => self.compactness,
            1 => self.partisan_deviation,
            2 => self.vra_deficit,
            _ => panic!("objective index out of bounds"),
        }
    }
}

#[test]
fn l1_three_objective_redistricting_like_fixture_sorts_fronts() {
    let candidates = vec![
        Candidate {
            compactness: 10.0,
            partisan_deviation: 1.0,
            vra_deficit: 0.0,
        },
        Candidate {
            compactness: 12.0,
            partisan_deviation: 1.2,
            vra_deficit: 0.2,
        },
        Candidate {
            compactness: 8.0,
            partisan_deviation: 2.0,
            vra_deficit: 0.0,
        },
    ];

    assert!(dominates(&candidates[0], &candidates[1]).unwrap());
    assert_eq!(
        fast_non_dominated_sort(&candidates).unwrap(),
        vec![vec![0, 2], vec![1]]
    );
}

#[test]
fn l1_crowding_uses_front_local_positions() {
    let candidates = vec![
        Candidate {
            compactness: 1.0,
            partisan_deviation: 1.0,
            vra_deficit: 1.0,
        },
        Candidate {
            compactness: 2.0,
            partisan_deviation: 2.0,
            vra_deficit: 2.0,
        },
        Candidate {
            compactness: 3.0,
            partisan_deviation: 3.0,
            vra_deficit: 3.0,
        },
    ];

    let distances = crowding_distance(&[0, 1, 2], &candidates).unwrap();

    assert!(distances[0].is_infinite());
    assert!(distances[1].is_finite());
    assert!(distances[2].is_infinite());
}

#[test]
fn l1_crowding_rejects_non_finite_distance_intermediates() {
    let candidates = vec![
        Candidate {
            compactness: -f64::MAX,
            partisan_deviation: 0.0,
            vra_deficit: 0.0,
        },
        Candidate {
            compactness: 0.0,
            partisan_deviation: 0.0,
            vra_deficit: 0.0,
        },
        Candidate {
            compactness: f64::MAX,
            partisan_deviation: 0.0,
            vra_deficit: 0.0,
        },
    ];

    match crowding_distance(&[0, 1, 2], &candidates) {
        Err(RoptError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "crowding objective range");
            assert!(value.is_infinite());
        }
        other => panic!("expected crowding overflow error, got {other:?}"),
    }
}

#[test]
fn l1_seed_derivation_matches_pareto_legacy_shape() {
    let init = derive_seed(b"PARETO_INIT_", &[SeedPart::U32(5), SeedPart::U64(99)]).unwrap();
    let cross = derive_seed(
        b"PARETO_CROSS_",
        &[SeedPart::U32(3), SeedPart::U32(7), SeedPart::U64(99)],
    )
    .unwrap();
    let mutation = derive_seed(
        b"PARETO_MUT_",
        &[SeedPart::U32(3), SeedPart::U32(7), SeedPart::U64(99)],
    )
    .unwrap();

    assert_eq!(
        init,
        derive_seed(b"PARETO_INIT_", &[SeedPart::U32(5), SeedPart::U64(99)]).unwrap()
    );
    assert_ne!(cross, mutation);
}

#[test]
fn l1_crowding_rejects_duplicate_front_membership() {
    let candidates = vec![
        Candidate {
            compactness: 1.0,
            partisan_deviation: 1.0,
            vra_deficit: 1.0,
        },
        Candidate {
            compactness: 2.0,
            partisan_deviation: 2.0,
            vra_deficit: 2.0,
        },
    ];

    assert_eq!(
        crowding_distance(&[0, 1, 0], &candidates),
        Err(RoptError::DuplicateFrontIndex {
            front_index: 2,
            point_index: 0
        })
    );
}
