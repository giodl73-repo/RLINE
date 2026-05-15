use rstat_core::hypothesis::{benjamini_hochberg, holm_bonferroni};
use rstat_core::mcmc::{effective_sample_size, hamming_autocorrelation};
use rstat_core::probability::{regularized_incomplete_beta, standard_normal_cdf};
use rstat_core::resampling::bootstrap_percentile_interval;
use rstat_core::summary::{quantile_sorted_copy, summary_stats, weighted_summary_stats};

#[test]
#[ignore = "L2 numeric stress: larger deterministic sample than normal unit suite"]
fn l2_large_summary_sample_is_stable() {
    let values: Vec<f64> = (0..100_000).map(|i| (i % 997) as f64 / 997.0).collect();

    let stats = summary_stats(&values).unwrap();
    let q99 = quantile_sorted_copy(&values, 0.99).unwrap();

    assert_eq!(stats.count, 100_000);
    assert!(stats.mean > 0.49 && stats.mean < 0.51);
    assert!(q99 > 0.98 && q99 < 1.0);
}

#[test]
#[ignore = "L2 numeric stress: larger weighted sample"]
fn l2_large_weighted_summary_sample_is_stable() {
    let values: Vec<f64> = (0..100_000).map(|i| (i % 997) as f64 / 997.0).collect();
    let weights: Vec<f64> = (0..100_000).map(|i| (1 + (i % 17)) as f64).collect();

    let stats = weighted_summary_stats(&values, &weights).unwrap();

    assert_eq!(stats.count, 100_000);
    assert!(stats.total_weight > 0.0);
    assert!(stats.mean > 0.49 && stats.mean < 0.51);
    assert!(stats.std_dev_population > 0.0);
}

#[test]
#[ignore = "L2 numeric stress: long autocorrelated trace"]
fn l2_long_autocorrelated_trace_has_reduced_ess() {
    let mut trace = Vec::with_capacity(20_000);
    let mut x = 0.0;
    for i in 0..20_000 {
        let noise = ((i * 48271 % 997) as f64 / 997.0) - 0.5;
        x = 0.97 * x + noise;
        trace.push(x);
    }

    let ess = effective_sample_size(&trace).unwrap();

    assert!(ess > 0.0);
    assert!(ess < trace.len() as f64 / 5.0);
}

#[test]
#[ignore = "L2 numeric stress: broad beta parameter smoke"]
fn l2_beta_cdf_remains_bounded_across_grid() {
    for a in [0.5, 1.0, 2.0, 10.0, 50.0] {
        for b in [0.5, 1.0, 3.0, 9.0, 40.0] {
            for x in [0.01, 0.10, 0.50, 0.90, 0.99] {
                let value = regularized_incomplete_beta(x, a, b).unwrap();
                assert!(
                    (0.0..=1.0).contains(&value),
                    "I_x({a},{b}) at {x} = {value}"
                );
            }
        }
    }
}

#[test]
#[ignore = "L2 numeric stress: broad normal CDF grid"]
fn l2_normal_cdf_grid_is_bounded_and_monotone() {
    let mut previous = 0.0;
    for i in -800..=800 {
        let z = i as f64 / 100.0;
        let value = standard_normal_cdf(z).unwrap();
        assert!((0.0..=1.0).contains(&value), "Phi({z}) = {value}");
        assert!(
            value + 1e-12 >= previous,
            "Phi grid must be monotone at {z}"
        );
        previous = value;
    }
    assert!(standard_normal_cdf(-8.0).unwrap() < 1e-14);
    assert!(standard_normal_cdf(8.0).unwrap() > 1.0 - 1e-14);
}

#[test]
#[ignore = "L2 numeric stress: larger partition trajectory"]
fn l2_hamming_autocorrelation_handles_large_trajectory() {
    let partitions: Vec<Vec<usize>> = (0..500)
        .map(|t| (0..200).map(|u| (u + t) % 13).collect())
        .collect();

    let autocorr = hamming_autocorrelation(&partitions, 50).unwrap();

    assert_eq!(autocorr.len(), 51);
    assert_eq!(autocorr[0], 0.0);
    assert!(autocorr.iter().all(|v| (0.0..=1.0).contains(v)));
}

#[test]
#[ignore = "L2 numeric stress: many deterministic bootstrap replicates"]
fn l2_bootstrap_percentile_interval_large_replicate_count() {
    let sample: Vec<f64> = (0..2_000).map(|i| (i % 101) as f64).collect();
    let stat = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;

    let (lo, hi) = bootstrap_percentile_interval(&sample, 5_000, 20260514, stat, 0.025, 0.975)
        .expect("large bootstrap should compute");

    assert!(lo < hi);
    assert!(lo > 45.0);
    assert!(hi < 55.0);
}

#[test]
#[ignore = "L2 numeric stress: large multiple-testing family"]
fn l2_multiple_testing_large_family_is_bounded_and_monotone() {
    let raw: Vec<f64> = (1..=10_000)
        .map(|i| (i as f64 / 10_000.0).powi(2))
        .collect();

    let holm = holm_bonferroni(&raw).unwrap();
    let bh = benjamini_hochberg(&raw).unwrap();

    assert_eq!(holm.len(), raw.len());
    assert_eq!(bh.len(), raw.len());
    assert!(holm.iter().all(|p| (0.0..=1.0).contains(p)));
    assert!(bh.iter().all(|p| (0.0..=1.0).contains(p)));
    for i in 1..holm.len() {
        assert!(holm[i] + 1e-12 >= holm[i - 1]);
        assert!(bh[i] + 1e-12 >= bh[i - 1]);
    }
}
