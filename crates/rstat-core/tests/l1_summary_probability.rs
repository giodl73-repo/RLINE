use rstat_core::hypothesis::{
    bayesian_detection_score, benjamini_hochberg, empirical_p_value, holm_bonferroni,
    holm_bonferroni_named, HypothesisError, Tail,
};
use rstat_core::probability::{regularized_incomplete_beta, standard_normal_cdf, ProbabilityError};
use rstat_core::resampling::{bootstrap_percentile_interval, bootstrap_statistics};
use rstat_core::summary::{
    percentile_interval_sorted_copy, quantile_sorted_copy, summary_stats, weighted_mean,
    weighted_summary_stats, SummaryError,
};

#[test]
fn l1_summary_quantiles_are_order_invariant() {
    let ordered = [1.0, 2.0, 3.0, 4.0, 5.0];
    let shuffled = [5.0, 1.0, 4.0, 2.0, 3.0];

    assert_eq!(
        quantile_sorted_copy(&ordered, 0.25).unwrap(),
        quantile_sorted_copy(&shuffled, 0.25).unwrap()
    );
    assert_eq!(
        percentile_interval_sorted_copy(&ordered, 0.025, 0.975).unwrap(),
        percentile_interval_sorted_copy(&shuffled, 0.025, 0.975).unwrap()
    );
}

#[test]
fn l1_quantile_rejects_non_finite_interpolation_result() {
    match quantile_sorted_copy(&[-f64::MAX, f64::MAX], 0.5) {
        Err(SummaryError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "quantile span");
            assert!(value.is_infinite());
        }
        other => panic!("expected quantile overflow error, got {other:?}"),
    }
}

#[test]
fn l1_summary_and_probability_compose_for_interval_report() {
    let samples = [0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80];

    let stats = summary_stats(&samples).unwrap();
    let (lo, hi) = percentile_interval_sorted_copy(&samples, 0.25, 0.75).unwrap();
    let beta_mid = regularized_incomplete_beta(0.5, 2.0, 2.0).unwrap();

    assert_eq!(stats.count, 8);
    assert!((stats.mean - 0.45).abs() < 1e-12);
    assert!(lo < stats.mean && stats.mean < hi);
    assert!((beta_mid - 0.5).abs() < 0.01);
}

#[test]
fn l1_normal_cdf_two_sided_p_value_composes_for_z_score() {
    let z = 1.96_f64;
    let p_two_sided = 2.0 * (1.0 - standard_normal_cdf(z.abs()).unwrap());

    assert!((p_two_sided - 0.0499958).abs() < 5e-6);
}

#[test]
fn l1_normal_cdf_rejects_non_finite_evidence_statistic() {
    assert_eq!(
        standard_normal_cdf(f64::NEG_INFINITY),
        Err(ProbabilityError::NonFiniteZ(f64::NEG_INFINITY))
    );
}

#[test]
fn l1_beta_cdf_rejects_overflowed_shape_aggregates() {
    match regularized_incomplete_beta(0.5, f64::MAX, f64::MAX) {
        Err(ProbabilityError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "beta shape window");
            assert!(value.is_infinite());
        }
        other => panic!("expected beta shape overflow error, got {other:?}"),
    }
}

#[test]
fn l1_weighted_summary_matches_expanded_sample() {
    let values = [1.0, 3.0, 5.0];
    let weights = [1.0, 2.0, 1.0];
    let expanded = [1.0, 3.0, 3.0, 5.0];

    let weighted = weighted_summary_stats(&values, &weights).unwrap();
    let expanded_stats = summary_stats(&expanded).unwrap();

    assert_eq!(weighted.mean, expanded_stats.mean);
    assert_eq!(
        weighted.variance_population,
        expanded_stats.variance_population
    );
    assert_eq!(
        weighted_mean(&values, &weights).unwrap(),
        expanded_stats.mean
    );
}

#[test]
fn l1_summary_rejects_non_finite_aggregate_results() {
    match summary_stats(&[f64::MAX, -f64::MAX]) {
        Err(SummaryError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "summary variance sum");
            assert!(value.is_infinite());
        }
        other => panic!("expected summary overflow error, got {other:?}"),
    }

    match weighted_mean(&[1.0, 1.0], &[f64::MAX, f64::MAX]) {
        Err(SummaryError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "weighted total weight");
            assert!(value.is_infinite());
        }
        other => panic!("expected weighted summary overflow error, got {other:?}"),
    }
}

#[test]
fn l1_bootstrap_summary_interval_composes_with_quantiles() {
    let sample = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let stat = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;

    let stats = bootstrap_statistics(&sample, 100, 99, stat).unwrap();
    let direct_interval = percentile_interval_sorted_copy(&stats, 0.10, 0.90).unwrap();
    let helper_interval =
        bootstrap_percentile_interval(&sample, 100, 99, stat, 0.10, 0.90).unwrap();

    assert_eq!(direct_interval, helper_interval);
    assert!(helper_interval.0 <= helper_interval.1);
}

#[test]
fn l1_percentile_interval_rejects_reversed_evidence_bounds() {
    let sample = [0.10, 0.20, 0.30, 0.40];

    assert_eq!(
        percentile_interval_sorted_copy(&sample, 0.80, 0.20),
        Err(SummaryError::InvalidIntervalQuantiles {
            low: 0.80,
            high: 0.20
        })
    );
}

#[test]
fn l1_empirical_p_value_and_detection_score_compose() {
    let reference: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();

    let (n_extreme, n_total, p_raw) = empirical_p_value(0.003, &reference, Tail::Lower).unwrap();
    let bds = bayesian_detection_score(0.05, p_raw, 70.0).unwrap();

    assert_eq!((n_extreme, n_total), (1, 100));
    assert!((p_raw - 0.01).abs() < 1e-12);
    assert!(bds > 0.80);
}

#[test]
fn l1_detection_score_rejects_overflowed_beta_shapes() {
    match bayesian_detection_score(0.5, 0.5, f64::MAX) {
        Err(HypothesisError::Probability(ProbabilityError::NonFiniteResult {
            operation,
            value,
        })) => {
            assert_eq!(operation, "beta lgamma(a)");
            assert!(value.is_infinite());
        }
        other => panic!("expected beta shape overflow error, got {other:?}"),
    }
}

#[test]
fn l1_multiple_testing_corrections_preserve_shape() {
    let raw = [0.001, 0.02, 0.03, 0.90];
    let holm = holm_bonferroni(&raw).unwrap();
    let bh = benjamini_hochberg(&raw).unwrap();

    assert_eq!(holm.len(), raw.len());
    assert_eq!(bh.len(), raw.len());
    assert!(holm.iter().all(|p| (0.0..=1.0).contains(p)));
    assert!(bh.iter().all(|p| (0.0..=1.0).contains(p)));
    assert!(holm[1] >= raw[1]);
    assert!(bh[1] <= holm[1]);
}

#[test]
fn l1_named_holm_rejects_duplicate_evidence_keys() {
    let raw = vec![
        ("candidate_a::primary".to_string(), 0.01),
        ("candidate_a::primary".to_string(), 0.04),
    ];

    assert_eq!(
        holm_bonferroni_named(&raw),
        Err(HypothesisError::DuplicateTestName(
            "candidate_a::primary".to_string()
        ))
    );
}
