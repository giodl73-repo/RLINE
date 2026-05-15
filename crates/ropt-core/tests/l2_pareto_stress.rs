use ropt_core::{crowding_distance, derive_seed, fast_non_dominated_sort, SeedPart};

#[test]
#[ignore = "L2 optimizer stress: larger deterministic Pareto cloud"]
fn l2_deterministic_pareto_cloud_sorts_and_crowds() {
    let objectives: Vec<[f64; 3]> = (0..500)
        .map(|i| {
            let x = i as f64;
            [x % 31.0, (499.0 - x) % 37.0, (x * 17.0) % 43.0]
        })
        .collect();

    let fronts = fast_non_dominated_sort(&objectives).unwrap();
    assert!(!fronts.is_empty());
    assert_eq!(fronts.iter().map(Vec::len).sum::<usize>(), objectives.len());

    let distances = crowding_distance(&fronts[0], &objectives).unwrap();
    assert_eq!(distances.len(), fronts[0].len());
}

#[test]
#[ignore = "L2 optimizer stress: long deterministic seed stream"]
fn l2_long_seed_stream_has_no_adjacent_collisions() {
    let mut previous =
        derive_seed(b"ROPT_STRESS_", &[SeedPart::U32(0), SeedPart::U64(42)]).unwrap();
    for index in 1..10_000_u32 {
        let next =
            derive_seed(b"ROPT_STRESS_", &[SeedPart::U32(index), SeedPart::U64(42)]).unwrap();
        assert_ne!(previous, next);
        previous = next;
    }
}
