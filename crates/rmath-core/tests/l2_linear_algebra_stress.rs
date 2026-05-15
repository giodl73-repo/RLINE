use rmath_core::{
    center_in_place, invert, l2_norm, mat_mul, normalize_in_place, symmetric_2x2_eigensystem,
    DenseMatrix, Symmetric2x2,
};

#[test]
#[ignore = "L2 numeric stress: Hilbert-like inverse smoke test"]
fn l2_hilbert_like_inverse_multiplies_to_identity() {
    let n = 8;
    let data: Vec<f64> = (0..n)
        .flat_map(|i| (0..n).map(move |j| 1.0 / ((i + j + 1) as f64)))
        .collect();
    let matrix = DenseMatrix::from_row_major(n, n, data).unwrap();

    let inv = invert(&matrix).unwrap();
    let identity = mat_mul(&matrix, &inv).unwrap();

    for i in 0..n {
        for j in 0..n {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!((identity.at(i, j) - expected).abs() < 1e-6);
        }
    }
}

#[test]
#[ignore = "L2 numeric stress: long alternating vector centering"]
fn l2_long_alternating_vector_centers_and_normalizes() {
    let mut values: Vec<f64> = (0..10_000)
        .map(|idx| {
            if idx % 2 == 0 {
                idx as f64
            } else {
                -(idx as f64)
            }
        })
        .collect();

    center_in_place(&mut values).unwrap();
    normalize_in_place(&mut values, 1e-14).unwrap();

    assert!(values.iter().sum::<f64>().abs() < 1e-9);
    assert!((l2_norm(&values).unwrap() - 1.0).abs() < 1e-12);
}

#[test]
#[ignore = "L2 numeric stress: near-degenerate symmetric 2x2 eigensystem"]
fn l2_near_degenerate_symmetric_2x2_has_unit_minor_vector() {
    let eigen = symmetric_2x2_eigensystem(Symmetric2x2 {
        a00: 1.0,
        a01: 1e-13,
        a11: 1.0 + 1e-13,
    })
    .unwrap();

    let norm = l2_norm(&[eigen.minor_eigenvector.0, eigen.minor_eigenvector.1]).unwrap();
    assert!(eigen.lambda_min <= eigen.lambda_max);
    assert!((norm - 1.0).abs() < 1e-12);
}
