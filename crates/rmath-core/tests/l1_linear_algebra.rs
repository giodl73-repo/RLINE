use rmath_core::{
    center_in_place, dot, invert, l2_norm, mat_mul, mat_mul_vec, normalize_centered,
    symmetric_2x2_eigensystem, transpose, DenseMatrix, LinearAlgebraError, Symmetric2x2,
};

#[test]
fn l1_solve_wls_normal_equation_fixture() {
    let x =
        DenseMatrix::from_row_major(4, 2, vec![1.0, 0.0, 1.0, 1.0, 1.0, 2.0, 1.0, 3.0]).unwrap();
    let y = [1.0, 3.0, 5.0, 7.0];
    let xt = transpose(&x).unwrap();
    let xtx = mat_mul(&xt, &x).unwrap();
    let xty = mat_mul_vec(&xt, &y).unwrap();
    let beta = mat_mul_vec(&invert(&xtx).unwrap(), &xty).unwrap();

    assert!((beta[0] - 1.0).abs() < 1e-12);
    assert!((beta[1] - 2.0).abs() < 1e-12);
}

#[test]
fn l1_partial_pivoting_handles_zero_initial_pivot() {
    let matrix = DenseMatrix::from_row_major(2, 2, vec![0.0, 2.0, 1.0, 3.0]).unwrap();

    let inv = invert(&matrix).unwrap();
    let identity = mat_mul(&matrix, &inv).unwrap();

    assert!((identity.at(0, 0) - 1.0).abs() < 1e-12);
    assert!(identity.at(0, 1).abs() < 1e-12);
    assert!(identity.at(1, 0).abs() < 1e-12);
    assert!((identity.at(1, 1) - 1.0).abs() < 1e-12);
}

#[test]
fn l1_inverse_overflow_is_rejected_before_non_finite_matrix_output() {
    let matrix = DenseMatrix::from_row_major(2, 2, vec![1e-11, f64::MAX, 0.0, 1.0]).unwrap();

    match invert(&matrix) {
        Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "invert row normalization");
            assert!(value.is_infinite());
        }
        other => panic!("expected inverse overflow error, got {other:?}"),
    }
}

#[test]
fn l1_matrix_product_overflow_is_rejected() {
    let a = DenseMatrix::from_row_major(1, 2, vec![f64::MAX, f64::MAX]).unwrap();
    let b = DenseMatrix::from_row_major(2, 1, vec![1.0, 1.0]).unwrap();

    match mat_mul(&a, &b) {
        Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "mat_mul");
            assert!(value.is_infinite());
        }
        other => panic!("expected matrix product overflow error, got {other:?}"),
    }
}

#[test]
fn l1_matrix_vector_product_overflow_is_rejected() {
    let a = DenseMatrix::from_row_major(1, 2, vec![f64::MAX, f64::MAX]).unwrap();

    match mat_mul_vec(&a, &[1.0, 1.0]) {
        Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "mat_mul_vec");
            assert!(value.is_infinite());
        }
        other => panic!("expected matrix-vector product overflow error, got {other:?}"),
    }
}

#[test]
fn l1_centered_normalization_matches_spectral_fixture() {
    let vector = normalize_centered(vec![-2.5, -1.5, -0.5, 0.5, 1.5, 2.5], 0.0).unwrap();

    assert!(vector.iter().sum::<f64>().abs() < 1e-12);
    assert!((l2_norm(&vector).unwrap() - 1.0).abs() < 1e-12);
    assert_eq!(vector[0].total_cmp(&vector[1]), std::cmp::Ordering::Less);
    assert_eq!(vector[4].total_cmp(&vector[5]), std::cmp::Ordering::Less);
}

#[test]
fn l1_projection_out_of_ones_matches_fiedler_fixture() {
    let mut vector = vec![
        (1.0_f64).sin(),
        (2.0_f64).sin(),
        (3.0_f64).sin(),
        (4.0_f64).sin(),
    ];

    center_in_place(&mut vector).unwrap();
    let ones = vec![1.0; vector.len()];

    assert!(dot(&vector, &ones).unwrap().abs() < 1e-12);
}

#[test]
fn l1_centering_rejects_overflowed_sum_policy() {
    let mut vector = vec![f64::MAX, f64::MAX];

    match center_in_place(&mut vector) {
        Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "center_in_place sum");
            assert!(value.is_infinite());
        }
        other => panic!("expected centering overflow error, got {other:?}"),
    }
    assert_eq!(vector, vec![f64::MAX, f64::MAX]);
}

#[test]
fn l1_non_finite_vector_input_is_rejected() {
    match l2_norm(&[1.0, f64::NAN]) {
        Err(LinearAlgebraError::NonFiniteValue { row, col, value }) => {
            assert_eq!((row, col), (1, 0));
            assert!(value.is_nan());
        }
        other => panic!("expected non-finite vector error, got {other:?}"),
    }
}

#[test]
fn l1_normalization_rejects_negative_min_norm_policy() {
    let result = normalize_centered(vec![1.0, 1.0, 1.0], -1e-12);

    assert_eq!(
        result,
        Err(LinearAlgebraError::NegativeNormalizationThreshold { value: -1e-12 })
    );
}

#[test]
fn l1_normalization_rejects_overflowed_norm_policy() {
    let result = normalize_centered(vec![f64::MAX, -f64::MAX], 0.0);

    match result {
        Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "dot");
            assert!(value.is_infinite());
        }
        other => panic!("expected norm overflow error, got {other:?}"),
    }
}

#[test]
fn l1_symmetric_2x2_matches_geosection_horizontal_band_covariance() {
    let eigen = symmetric_2x2_eigensystem(Symmetric2x2 {
        a00: 0.0025,
        a01: 0.0,
        a11: 8.25,
    })
    .unwrap();

    assert!((eigen.lambda_min - 0.0025).abs() < 1e-12);
    assert_eq!(eigen.minor_eigenvector, (1.0, 0.0));
    assert!(
        (eigen
            .minor_eigenvector
            .0
            .atan2(eigen.minor_eigenvector.1)
            .to_degrees()
            - 90.0)
            .abs()
            < 1e-12
    );
}

#[test]
fn l1_symmetric_2x2_rejects_overflowed_covariance_result() {
    let result = symmetric_2x2_eigensystem(Symmetric2x2 {
        a00: f64::MAX,
        a01: 0.0,
        a11: -f64::MAX,
    });

    match result {
        Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
            assert_eq!(operation, "symmetric_2x2_eigensystem eigenvalue");
            assert!(value.is_infinite());
        }
        other => panic!("expected non-finite eigensystem result, got {other:?}"),
    }
}
