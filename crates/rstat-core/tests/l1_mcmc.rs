use rstat_core::mcmc::{
    effective_sample_size, gelman_rubin_rhat, hamming_autocorrelation,
    integrated_autocorrelation_time, DiagnosticsError,
};

#[test]
fn l1_mcmc_diagnostics_separate_converged_and_stuck_traces() {
    let c1 = vec![0.49, 0.50, 0.51, 0.50, 0.49, 0.51];
    let c2 = vec![0.50, 0.49, 0.51, 0.50, 0.51, 0.49];
    let c3 = vec![0.51, 0.50, 0.49, 0.50, 0.49, 0.51];
    let c4 = vec![0.50, 0.51, 0.49, 0.50, 0.51, 0.49];
    let chains = vec![c1.as_slice(), c2.as_slice(), c3.as_slice(), c4.as_slice()];

    let rhat = gelman_rubin_rhat(&chains).unwrap();
    let mixed_ess = effective_sample_size(&[0.0, 1.0, -1.0, 0.5, -0.5, 1.5, -1.5, 0.25]).unwrap();
    let stuck_ess = effective_sample_size(&[1.0; 8]).unwrap();

    assert!(rhat < 1.05);
    assert!(mixed_ess > 0.0);
    assert_eq!(stuck_ess, 8.0);
}

#[test]
fn l1_rhat_rejects_non_finite_evidence_trace() {
    let c1 = vec![0.49, 0.50, 0.51, 0.50];
    let c2 = vec![0.50, 0.49, f64::INFINITY, 0.50];
    let c3 = vec![0.51, 0.50, 0.49, 0.50];
    let c4 = vec![0.50, 0.51, 0.49, 0.50];
    let chains = vec![c1.as_slice(), c2.as_slice(), c3.as_slice(), c4.as_slice()];

    assert_eq!(
        gelman_rubin_rhat(&chains),
        Err(DiagnosticsError::NonFiniteChainValue {
            chain_index: 1,
            sample_index: 2,
            value: f64::INFINITY
        })
    );
}

#[test]
fn l1_ess_rejects_non_finite_evidence_trace() {
    match effective_sample_size(&[0.1, 0.2, f64::NAN, 0.4]) {
        Err(DiagnosticsError::NonFiniteTraceValue { index, value }) => {
            assert_eq!(index, 2);
            assert!(value.is_nan());
        }
        other => panic!("expected non-finite trace value error, got {other:?}"),
    }
}

#[test]
fn l1_mcmc_rejects_non_finite_diagnostic_aggregates() {
    let c1 = vec![f64::MAX, -f64::MAX];
    let c2 = vec![1.0, 1.0];
    let c3 = vec![1.0, 1.0];
    let c4 = vec![1.0, 1.0];
    let chains = vec![c1.as_slice(), c2.as_slice(), c3.as_slice(), c4.as_slice()];

    match gelman_rubin_rhat(&chains) {
        Err(DiagnosticsError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "rhat chain variance sum");
            assert!(value.is_infinite());
        }
        other => panic!("expected rhat overflow error, got {other:?}"),
    }

    match effective_sample_size(&[f64::MAX, -f64::MAX, f64::MAX, -f64::MAX]) {
        Err(DiagnosticsError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "ess variance sum");
            assert!(value.is_infinite());
        }
        other => panic!("expected ess overflow error, got {other:?}"),
    }
}

#[test]
fn l1_hamming_tau_tracks_mixing_speed() {
    let slow = vec![
        vec![1, 1, 2, 2],
        vec![1, 1, 2, 2],
        vec![1, 2, 2, 2],
        vec![2, 2, 2, 1],
    ];
    let fast = vec![
        vec![1, 1, 1, 1],
        vec![2, 2, 2, 2],
        vec![1, 1, 1, 1],
        vec![2, 2, 2, 2],
    ];

    let slow_h = hamming_autocorrelation(&slow, 3).unwrap();
    let fast_h = hamming_autocorrelation(&fast, 3).unwrap();

    let slow_tau = integrated_autocorrelation_time(&slow_h).unwrap();
    let fast_tau = integrated_autocorrelation_time(&fast_h).unwrap();

    assert!(slow_tau > 1.0);
    assert!(fast_tau <= slow_tau);
}

#[test]
fn l1_tau_rejects_invalid_hamming_lag_values() {
    assert_eq!(
        integrated_autocorrelation_time(&[0.0, -0.1]),
        Err(DiagnosticsError::InvalidAutocorrelationValue {
            lag: 1,
            value: -0.1
        })
    );
}

#[test]
fn l1_hamming_rejects_empty_partition_vectors() {
    let partitions = vec![Vec::new(), Vec::new(), Vec::new()];

    assert_eq!(
        hamming_autocorrelation(&partitions, 2),
        Err(DiagnosticsError::EmptyPartition(0))
    );
}
