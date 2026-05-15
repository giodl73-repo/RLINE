use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum LinearAlgebraError {
    #[error("[INPUT] matrix dimensions do not conform: left {left:?}, right {right:?}")]
    DimensionMismatch {
        left: (usize, usize),
        right: (usize, usize),
    },
    #[error("[INPUT] vector must not be empty")]
    EmptyVector,
    #[error("[INPUT] matrix must be square, got {rows}x{cols}")]
    NotSquare { rows: usize, cols: usize },
    #[error("[INPUT] matrix contains non-finite value {value} at ({row}, {col})")]
    NonFiniteValue { row: usize, col: usize, value: f64 },
    #[error("[INPUT] normalization threshold must be non-negative, got {value}")]
    NegativeNormalizationThreshold { value: f64 },
    #[error("[NUMERIC] {operation} produced non-finite value {value}")]
    NonFiniteResult { operation: &'static str, value: f64 },
    #[error("[NUMERIC] singular matrix")]
    Singular,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DenseMatrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Symmetric2x2 {
    pub a00: f64,
    pub a01: f64,
    pub a11: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Symmetric2x2Eigensystem {
    pub lambda_min: f64,
    pub lambda_max: f64,
    pub minor_eigenvector: (f64, f64),
}

impl DenseMatrix {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn from_row_major(
        rows: usize,
        cols: usize,
        data: Vec<f64>,
    ) -> Result<Self, LinearAlgebraError> {
        if data.len() != rows * cols {
            return Err(LinearAlgebraError::DimensionMismatch {
                left: (1, data.len()),
                right: (rows, cols),
            });
        }
        let matrix = Self { rows, cols, data };
        validate_finite(&matrix)?;
        Ok(matrix)
    }

    pub fn identity(n: usize) -> Self {
        let mut out = Self::zeros(n, n);
        for i in 0..n {
            out.set(i, i, 1.0);
        }
        out
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn at(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] = value;
    }
}

pub fn mat_mul(a: &DenseMatrix, b: &DenseMatrix) -> Result<DenseMatrix, LinearAlgebraError> {
    validate_finite(a)?;
    validate_finite(b)?;
    if a.cols != b.rows {
        return Err(LinearAlgebraError::DimensionMismatch {
            left: (a.rows, a.cols),
            right: (b.rows, b.cols),
        });
    }
    let mut out = DenseMatrix::zeros(a.rows, b.cols);
    for i in 0..a.rows {
        for k in 0..a.cols {
            let aik = a.at(i, k);
            if aik == 0.0 {
                continue;
            }
            for j in 0..b.cols {
                let value = out.at(i, j) + aik * b.at(k, j);
                validate_result_scalar("mat_mul", value)?;
                out.set(i, j, value);
            }
        }
    }
    Ok(out)
}

pub fn mat_mul_vec(a: &DenseMatrix, v: &[f64]) -> Result<Vec<f64>, LinearAlgebraError> {
    validate_finite(a)?;
    validate_vector(v)?;
    if a.cols != v.len() {
        return Err(LinearAlgebraError::DimensionMismatch {
            left: (a.rows, a.cols),
            right: (v.len(), 1),
        });
    }
    let mut out = vec![0.0; a.rows];
    for i in 0..a.rows {
        let mut sum = 0.0;
        for (j, value) in v.iter().enumerate().take(a.cols) {
            sum += a.at(i, j) * value;
            validate_result_scalar("mat_mul_vec", sum)?;
        }
        out[i] = sum;
    }
    Ok(out)
}

pub fn transpose(a: &DenseMatrix) -> Result<DenseMatrix, LinearAlgebraError> {
    validate_finite(a)?;
    let mut out = DenseMatrix::zeros(a.cols, a.rows);
    for i in 0..a.rows {
        for j in 0..a.cols {
            out.set(j, i, a.at(i, j));
        }
    }
    Ok(out)
}

pub fn invert(m: &DenseMatrix) -> Result<DenseMatrix, LinearAlgebraError> {
    validate_finite(m)?;
    if m.rows != m.cols {
        return Err(LinearAlgebraError::NotSquare {
            rows: m.rows,
            cols: m.cols,
        });
    }
    let n = m.rows;
    let mut aug = DenseMatrix::zeros(n, 2 * n);
    for i in 0..n {
        for j in 0..n {
            aug.set(i, j, m.at(i, j));
        }
        aug.set(i, n + i, 1.0);
    }
    for c in 0..n {
        let mut pivot_row = c;
        let mut pivot_abs = aug.at(c, c).abs();
        for r in (c + 1)..n {
            let value = aug.at(r, c).abs();
            if value > pivot_abs {
                pivot_abs = value;
                pivot_row = r;
            }
        }
        if pivot_abs < 1e-12 {
            return Err(LinearAlgebraError::Singular);
        }
        if pivot_row != c {
            for j in 0..(2 * n) {
                let tmp = aug.at(c, j);
                aug.set(c, j, aug.at(pivot_row, j));
                aug.set(pivot_row, j, tmp);
            }
        }
        let pivot = aug.at(c, c);
        for j in 0..(2 * n) {
            let value = aug.at(c, j) / pivot;
            validate_result_scalar("invert row normalization", value)?;
            aug.set(c, j, value);
        }
        for r in 0..n {
            if r == c {
                continue;
            }
            let factor = aug.at(r, c);
            if factor == 0.0 {
                continue;
            }
            for j in 0..(2 * n) {
                let value = aug.at(r, j) - factor * aug.at(c, j);
                validate_result_scalar("invert row elimination", value)?;
                aug.set(r, j, value);
            }
        }
    }

    let mut inv = DenseMatrix::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            let value = aug.at(i, n + j);
            validate_result_scalar("invert result", value)?;
            inv.set(i, j, value);
        }
    }
    Ok(inv)
}

pub fn dot(a: &[f64], b: &[f64]) -> Result<f64, LinearAlgebraError> {
    validate_vector(a)?;
    validate_vector(b)?;
    if a.len() != b.len() {
        return Err(LinearAlgebraError::DimensionMismatch {
            left: (a.len(), 1),
            right: (b.len(), 1),
        });
    }
    let value = a.iter().zip(b).map(|(x, y)| x * y).sum();
    validate_result_scalar("dot", value)?;
    Ok(value)
}

pub fn l2_norm(values: &[f64]) -> Result<f64, LinearAlgebraError> {
    let norm = dot(values, values)?.sqrt();
    validate_result_scalar("l2_norm", norm)?;
    Ok(norm)
}

pub fn center_in_place(values: &mut [f64]) -> Result<(), LinearAlgebraError> {
    validate_non_empty_vector(values)?;
    let sum = values.iter().sum::<f64>();
    validate_result_scalar("center_in_place sum", sum)?;
    let mean = sum / values.len() as f64;
    validate_result_scalar("center_in_place mean", mean)?;
    let mut centered = Vec::with_capacity(values.len());
    for &value in values.iter() {
        let next = value - mean;
        validate_result_scalar("center_in_place centered value", next)?;
        centered.push(next);
    }
    for (value, centered_value) in values.iter_mut().zip(centered) {
        *value = centered_value;
    }
    Ok(())
}

pub fn normalize_in_place(values: &mut [f64], min_norm: f64) -> Result<bool, LinearAlgebraError> {
    validate_vector(values)?;
    if !min_norm.is_finite() {
        return Err(LinearAlgebraError::NonFiniteValue {
            row: 0,
            col: 0,
            value: min_norm,
        });
    }
    if min_norm < 0.0 {
        return Err(LinearAlgebraError::NegativeNormalizationThreshold { value: min_norm });
    }
    let norm = l2_norm(values)?;
    if norm > min_norm {
        for value in values {
            *value /= norm;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn normalize_centered(
    mut values: Vec<f64>,
    min_norm: f64,
) -> Result<Vec<f64>, LinearAlgebraError> {
    center_in_place(&mut values)?;
    normalize_in_place(&mut values, min_norm)?;
    Ok(values)
}

pub fn symmetric_2x2_eigensystem(
    matrix: Symmetric2x2,
) -> Result<Symmetric2x2Eigensystem, LinearAlgebraError> {
    validate_scalar(matrix.a00, 0, 0)?;
    validate_scalar(matrix.a01, 0, 1)?;
    validate_scalar(matrix.a11, 1, 1)?;

    let trace_half = (matrix.a00 + matrix.a11) / 2.0;
    let disc = (((matrix.a00 - matrix.a11) / 2.0).powi(2) + matrix.a01.powi(2)).sqrt();
    let lambda_min = trace_half - disc;
    let lambda_max = trace_half + disc;
    validate_result_scalar("symmetric_2x2_eigensystem eigenvalue", lambda_min)?;
    validate_result_scalar("symmetric_2x2_eigensystem eigenvalue", lambda_max)?;

    let mut minor_eigenvector = if matrix.a01.abs() > 1e-12 {
        (matrix.a01, lambda_min - matrix.a00)
    } else if matrix.a00 < matrix.a11 {
        (1.0, 0.0)
    } else {
        (0.0, 1.0)
    };
    let norm = l2_norm(&[minor_eigenvector.0, minor_eigenvector.1])?;
    validate_result_scalar("symmetric_2x2_eigensystem eigenvector norm", norm)?;
    if norm > 0.0 {
        minor_eigenvector.0 /= norm;
        minor_eigenvector.1 /= norm;
    }

    Ok(Symmetric2x2Eigensystem {
        lambda_min,
        lambda_max,
        minor_eigenvector,
    })
}

fn validate_finite(matrix: &DenseMatrix) -> Result<(), LinearAlgebraError> {
    for row in 0..matrix.rows {
        for col in 0..matrix.cols {
            let value = matrix.at(row, col);
            validate_scalar(value, row, col)?;
        }
    }
    Ok(())
}

fn validate_vector(values: &[f64]) -> Result<(), LinearAlgebraError> {
    for (row, &value) in values.iter().enumerate() {
        validate_scalar(value, row, 0)?;
    }
    Ok(())
}

fn validate_scalar(value: f64, row: usize, col: usize) -> Result<(), LinearAlgebraError> {
    if !value.is_finite() {
        return Err(LinearAlgebraError::NonFiniteValue { row, col, value });
    }
    Ok(())
}

fn validate_result_scalar(operation: &'static str, value: f64) -> Result<(), LinearAlgebraError> {
    if !value.is_finite() {
        return Err(LinearAlgebraError::NonFiniteResult { operation, value });
    }
    Ok(())
}

fn validate_non_empty_vector(values: &[f64]) -> Result<(), LinearAlgebraError> {
    if values.is_empty() {
        return Err(LinearAlgebraError::EmptyVector);
    }
    validate_vector(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l0_matrix_multiply_matches_hand_computed_values() {
        let a = DenseMatrix::from_row_major(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        let b = DenseMatrix::from_row_major(3, 2, vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0]).unwrap();

        let product = mat_mul(&a, &b).unwrap();

        assert_eq!(
            product,
            DenseMatrix::from_row_major(2, 2, vec![58.0, 64.0, 139.0, 154.0]).unwrap()
        );
    }

    #[test]
    fn l0_matrix_multiply_overflow_is_rejected() {
        let a = DenseMatrix::from_row_major(1, 1, vec![f64::MAX]).unwrap();
        let b = DenseMatrix::from_row_major(1, 1, vec![2.0]).unwrap();

        match mat_mul(&a, &b) {
            Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
                assert_eq!(operation, "mat_mul");
                assert!(value.is_infinite());
            }
            other => panic!("expected mat_mul overflow error, got {other:?}"),
        }
    }

    #[test]
    fn l0_matrix_vector_multiply_matches_hand_computed_values() {
        let a = DenseMatrix::from_row_major(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();

        assert_eq!(
            mat_mul_vec(&a, &[10.0, 20.0, 30.0]).unwrap(),
            vec![140.0, 320.0]
        );
    }

    #[test]
    fn l0_matrix_vector_multiply_overflow_is_rejected() {
        let a = DenseMatrix::from_row_major(1, 1, vec![f64::MAX]).unwrap();

        match mat_mul_vec(&a, &[2.0]) {
            Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
                assert_eq!(operation, "mat_mul_vec");
                assert!(value.is_infinite());
            }
            other => panic!("expected mat_mul_vec overflow error, got {other:?}"),
        }
    }

    #[test]
    fn l0_transpose_swaps_axes() {
        let a = DenseMatrix::from_row_major(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        let t = transpose(&a).unwrap();

        assert_eq!(
            t,
            DenseMatrix::from_row_major(3, 2, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]).unwrap()
        );
    }

    #[test]
    fn l0_inverse_multiplies_back_to_identity() {
        let a = DenseMatrix::from_row_major(2, 2, vec![4.0, 7.0, 2.0, 6.0]).unwrap();
        let inv = invert(&a).unwrap();
        let identity = mat_mul(&a, &inv).unwrap();

        for i in 0..2 {
            for j in 0..2 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((identity.at(i, j) - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn l0_singular_matrix_is_rejected() {
        let a = DenseMatrix::from_row_major(2, 2, vec![1.0, 2.0, 2.0, 4.0]).unwrap();

        assert_eq!(invert(&a), Err(LinearAlgebraError::Singular));
    }

    #[test]
    fn l0_inverse_rejects_overflowed_row_normalization() {
        let a = DenseMatrix::from_row_major(2, 2, vec![1e-11, f64::MAX, 0.0, 1.0]).unwrap();

        match invert(&a) {
            Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
                assert_eq!(operation, "invert row normalization");
                assert!(value.is_infinite());
            }
            other => panic!("expected inverse normalization overflow error, got {other:?}"),
        }
    }

    #[test]
    fn l0_dimension_mismatch_is_rejected() {
        let a = DenseMatrix::zeros(2, 3);
        let b = DenseMatrix::zeros(2, 3);

        assert_eq!(
            mat_mul(&a, &b),
            Err(LinearAlgebraError::DimensionMismatch {
                left: (2, 3),
                right: (2, 3)
            })
        );
    }

    #[test]
    fn l0_dot_and_norm_match_hand_computed_values() {
        assert_eq!(dot(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]).unwrap(), 32.0);
        assert_eq!(l2_norm(&[3.0, 4.0]).unwrap(), 5.0);
    }

    #[test]
    fn l0_dot_overflow_is_rejected() {
        match dot(&[f64::MAX, f64::MAX], &[2.0, 2.0]) {
            Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
                assert_eq!(operation, "dot");
                assert!(value.is_infinite());
            }
            other => panic!("expected dot overflow error, got {other:?}"),
        }
    }

    #[test]
    fn l0_normalization_rejects_overflowed_norm_without_mutating() {
        let mut values = vec![f64::MAX, f64::MAX];

        match normalize_in_place(&mut values, 0.0) {
            Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
                assert_eq!(operation, "dot");
                assert!(value.is_infinite());
            }
            other => panic!("expected norm overflow error, got {other:?}"),
        }
        assert_eq!(values, vec![f64::MAX, f64::MAX]);
    }

    #[test]
    fn l0_centering_removes_mean() {
        let mut values = vec![2.0, 4.0, 6.0];
        center_in_place(&mut values).unwrap();

        assert_eq!(values, vec![-2.0, 0.0, 2.0]);
        assert!(values.iter().sum::<f64>().abs() < 1e-12);
    }

    #[test]
    fn l0_centering_rejects_overflowed_sum_without_mutating() {
        let mut values = vec![f64::MAX, f64::MAX];

        match center_in_place(&mut values) {
            Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
                assert_eq!(operation, "center_in_place sum");
                assert!(value.is_infinite());
            }
            other => panic!("expected centering overflow error, got {other:?}"),
        }
        assert_eq!(values, vec![f64::MAX, f64::MAX]);
    }

    #[test]
    fn l0_normalize_centered_returns_unit_centered_vector() {
        let values = normalize_centered(vec![1.0, 2.0, 3.0], 0.0).unwrap();

        assert!(values.iter().sum::<f64>().abs() < 1e-12);
        assert!((l2_norm(&values).unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn l0_near_zero_normalization_reports_unchanged() {
        let mut values = vec![0.0, 0.0, 0.0];

        assert!(!normalize_in_place(&mut values, 1e-14).unwrap());
        assert_eq!(values, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn l0_negative_normalization_threshold_is_rejected() {
        let mut values = vec![0.0, 0.0, 0.0];

        assert_eq!(
            normalize_in_place(&mut values, -1.0),
            Err(LinearAlgebraError::NegativeNormalizationThreshold { value: -1.0 })
        );
        assert_eq!(values, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn l0_vector_dimension_mismatch_is_rejected() {
        assert_eq!(
            dot(&[1.0, 2.0], &[1.0]),
            Err(LinearAlgebraError::DimensionMismatch {
                left: (2, 1),
                right: (1, 1)
            })
        );
    }

    #[test]
    fn l0_empty_centering_is_rejected() {
        let mut values = Vec::new();

        assert_eq!(
            center_in_place(&mut values),
            Err(LinearAlgebraError::EmptyVector)
        );
    }

    #[test]
    fn l0_symmetric_2x2_diagonal_minor_axis() {
        let eigen = symmetric_2x2_eigensystem(Symmetric2x2 {
            a00: 1.0,
            a01: 0.0,
            a11: 4.0,
        })
        .unwrap();

        assert_eq!(eigen.lambda_min, 1.0);
        assert_eq!(eigen.lambda_max, 4.0);
        assert_eq!(eigen.minor_eigenvector, (1.0, 0.0));
    }

    #[test]
    fn l0_symmetric_2x2_off_diagonal_eigenpair_matches_equation() {
        let matrix = Symmetric2x2 {
            a00: 2.0,
            a01: 1.0,
            a11: 2.0,
        };
        let eigen = symmetric_2x2_eigensystem(matrix).unwrap();
        let (x, y) = eigen.minor_eigenvector;

        assert!((eigen.lambda_min - 1.0).abs() < 1e-12);
        assert!((eigen.lambda_max - 3.0).abs() < 1e-12);
        assert!((matrix.a00 * x + matrix.a01 * y - eigen.lambda_min * x).abs() < 1e-12);
        assert!((matrix.a01 * x + matrix.a11 * y - eigen.lambda_min * y).abs() < 1e-12);
    }

    #[test]
    fn l0_symmetric_2x2_non_finite_is_rejected() {
        match symmetric_2x2_eigensystem(Symmetric2x2 {
            a00: 1.0,
            a01: f64::INFINITY,
            a11: 2.0,
        }) {
            Err(LinearAlgebraError::NonFiniteValue { row, col, value }) => {
                assert_eq!((row, col), (0, 1));
                assert!(value.is_infinite());
            }
            other => panic!("expected non-finite matrix error, got {other:?}"),
        }
    }

    #[test]
    fn l0_symmetric_2x2_rejects_non_finite_eigenvalue_result() {
        match symmetric_2x2_eigensystem(Symmetric2x2 {
            a00: f64::MAX,
            a01: 0.0,
            a11: -f64::MAX,
        }) {
            Err(LinearAlgebraError::NonFiniteResult { operation, value }) => {
                assert_eq!(operation, "symmetric_2x2_eigensystem eigenvalue");
                assert!(value.is_infinite());
            }
            other => panic!("expected non-finite eigensystem result, got {other:?}"),
        }
    }
}
