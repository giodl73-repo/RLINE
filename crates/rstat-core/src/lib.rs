pub mod summary {
    use thiserror::Error;

    #[derive(Debug, Error, Clone, PartialEq)]
    pub enum SummaryError {
        #[error("[INPUT] empty sample")]
        EmptySample,
        #[error("[INPUT] value and weight lengths differ: {values} values vs {weights} weights")]
        LengthMismatch { values: usize, weights: usize },
        #[error("[INPUT] sample contains non-finite value {value} at index {index}")]
        NonFiniteValue { index: usize, value: f64 },
        #[error("[INPUT] weight contains negative or non-finite value {value} at index {index}")]
        InvalidWeight { index: usize, value: f64 },
        #[error("[INPUT] total weight must be positive")]
        ZeroTotalWeight,
        #[error("[INPUT] quantile q must be in [0, 1], got {0}")]
        InvalidQuantile(f64),
        #[error("[INPUT] percentile interval quantiles must satisfy low <= high, got low={low}, high={high}")]
        InvalidIntervalQuantiles { low: f64, high: f64 },
        #[error("[NUMERIC] {operation} produced non-finite value {value}")]
        NonFiniteResult { operation: &'static str, value: f64 },
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct SummaryStats {
        pub count: usize,
        pub mean: f64,
        pub variance_population: f64,
        pub variance_sample: Option<f64>,
        pub std_dev_population: f64,
        pub std_dev_sample: Option<f64>,
        pub min: f64,
        pub max: f64,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct WeightedSummaryStats {
        pub count: usize,
        pub total_weight: f64,
        pub mean: f64,
        pub variance_population: f64,
        pub std_dev_population: f64,
        pub min: f64,
        pub max: f64,
    }

    pub fn mean(values: &[f64]) -> Result<f64, SummaryError> {
        validate_values(values)?;
        let sum = checked_sum(values, "mean sum")?;
        let mean = sum / values.len() as f64;
        validate_result("mean", mean)?;
        Ok(mean)
    }

    pub fn summary_stats(values: &[f64]) -> Result<SummaryStats, SummaryError> {
        validate_values(values)?;
        let count = values.len();
        let sum = checked_sum(values, "summary mean sum")?;
        let mean = sum / count as f64;
        validate_result("summary mean", mean)?;
        let mut sum_sq = 0.0;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for &value in values {
            sum_sq += (value - mean).powi(2);
            validate_result("summary variance sum", sum_sq)?;
            min = min.min(value);
            max = max.max(value);
        }
        let variance_population = sum_sq / count as f64;
        validate_result("summary population variance", variance_population)?;
        let variance_sample = if count > 1 {
            let variance = sum_sq / (count - 1) as f64;
            validate_result("summary sample variance", variance)?;
            Some(variance)
        } else {
            None
        };
        let std_dev_population = variance_population.sqrt();
        validate_result("summary population stddev", std_dev_population)?;
        let std_dev_sample = variance_sample
            .map(f64::sqrt)
            .map(|value| {
                validate_result("summary sample stddev", value)?;
                Ok(value)
            })
            .transpose()?;

        Ok(SummaryStats {
            count,
            mean,
            variance_population,
            variance_sample,
            std_dev_population,
            std_dev_sample,
            min,
            max,
        })
    }

    pub fn weighted_mean(values: &[f64], weights: &[f64]) -> Result<f64, SummaryError> {
        let (total_weight, weighted_sum) = validate_weighted_values(values, weights)?;
        let mean = weighted_sum / total_weight;
        validate_result("weighted mean", mean)?;
        Ok(mean)
    }

    pub fn weighted_summary_stats(
        values: &[f64],
        weights: &[f64],
    ) -> Result<WeightedSummaryStats, SummaryError> {
        let (total_weight, weighted_sum) = validate_weighted_values(values, weights)?;
        let mean = weighted_sum / total_weight;
        validate_result("weighted mean", mean)?;
        let mut weighted_sum_sq = 0.0;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for (&value, &weight) in values.iter().zip(weights) {
            weighted_sum_sq += weight * (value - mean).powi(2);
            validate_result("weighted variance sum", weighted_sum_sq)?;
            min = min.min(value);
            max = max.max(value);
        }
        let variance_population = weighted_sum_sq / total_weight;
        validate_result("weighted population variance", variance_population)?;
        let std_dev_population = variance_population.max(0.0).sqrt();
        validate_result("weighted population stddev", std_dev_population)?;

        Ok(WeightedSummaryStats {
            count: values.len(),
            total_weight,
            mean,
            variance_population,
            std_dev_population,
            min,
            max,
        })
    }

    pub fn weighted_std_dev_population(
        values: &[f64],
        weights: &[f64],
    ) -> Result<f64, SummaryError> {
        Ok(weighted_summary_stats(values, weights)?.std_dev_population)
    }

    pub fn median(values: &[f64]) -> Result<f64, SummaryError> {
        quantile_sorted_copy(values, 0.5)
    }

    /// Deterministic R-7 quantile, the default interpolation used by R and NumPy.
    pub fn quantile_sorted_copy(values: &[f64], q: f64) -> Result<f64, SummaryError> {
        validate_values(values)?;
        validate_quantile(q)?;
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        quantile_sorted(&sorted, q)
    }

    /// Deterministic R-7 quantile over an already sorted finite sample.
    pub fn quantile_sorted(sorted_values: &[f64], q: f64) -> Result<f64, SummaryError> {
        validate_values(sorted_values)?;
        validate_quantile(q)?;
        if sorted_values.len() == 1 {
            return Ok(sorted_values[0]);
        }

        let h = (sorted_values.len() - 1) as f64 * q;
        let lo = h.floor() as usize;
        let hi = h.ceil() as usize;
        if lo == hi {
            Ok(sorted_values[lo])
        } else {
            let frac = h - lo as f64;
            let span = sorted_values[hi] - sorted_values[lo];
            validate_result("quantile span", span)?;
            let increment = span * frac;
            validate_result("quantile increment", increment)?;
            let quantile = sorted_values[lo] + increment;
            validate_result("quantile interpolation", quantile)?;
            Ok(quantile)
        }
    }

    pub fn percentile_interval_sorted_copy(
        values: &[f64],
        low_q: f64,
        high_q: f64,
    ) -> Result<(f64, f64), SummaryError> {
        validate_values(values)?;
        validate_quantile(low_q)?;
        validate_quantile(high_q)?;
        validate_interval_quantiles(low_q, high_q)?;
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        Ok((
            quantile_sorted(&sorted, low_q)?,
            quantile_sorted(&sorted, high_q)?,
        ))
    }

    fn validate_values(values: &[f64]) -> Result<(), SummaryError> {
        if values.is_empty() {
            return Err(SummaryError::EmptySample);
        }
        for (index, &value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(SummaryError::NonFiniteValue { index, value });
            }
        }
        Ok(())
    }

    fn validate_weighted_values(
        values: &[f64],
        weights: &[f64],
    ) -> Result<(f64, f64), SummaryError> {
        validate_values(values)?;
        if values.len() != weights.len() {
            return Err(SummaryError::LengthMismatch {
                values: values.len(),
                weights: weights.len(),
            });
        }

        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        for (index, (&value, &weight)) in values.iter().zip(weights).enumerate() {
            if !weight.is_finite() || weight < 0.0 {
                return Err(SummaryError::InvalidWeight {
                    index,
                    value: weight,
                });
            }
            total_weight += weight;
            validate_result("weighted total weight", total_weight)?;
            weighted_sum += value * weight;
            validate_result("weighted sum", weighted_sum)?;
        }
        if total_weight <= 0.0 {
            return Err(SummaryError::ZeroTotalWeight);
        }
        Ok((total_weight, weighted_sum))
    }

    fn checked_sum(values: &[f64], operation: &'static str) -> Result<f64, SummaryError> {
        let mut sum = 0.0;
        for &value in values {
            sum += value;
            validate_result(operation, sum)?;
        }
        Ok(sum)
    }

    fn validate_result(operation: &'static str, value: f64) -> Result<(), SummaryError> {
        if !value.is_finite() {
            return Err(SummaryError::NonFiniteResult { operation, value });
        }
        Ok(())
    }

    fn validate_quantile(q: f64) -> Result<(), SummaryError> {
        if !q.is_finite() || !(0.0..=1.0).contains(&q) {
            return Err(SummaryError::InvalidQuantile(q));
        }
        Ok(())
    }

    fn validate_interval_quantiles(low_q: f64, high_q: f64) -> Result<(), SummaryError> {
        if low_q > high_q {
            return Err(SummaryError::InvalidIntervalQuantiles {
                low: low_q,
                high: high_q,
            });
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn l0_summary_stats_match_hand_computed_values() {
            let stats = summary_stats(&[1.0, 2.0, 3.0, 4.0]).unwrap();
            assert_eq!(stats.count, 4);
            assert_eq!(stats.mean, 2.5);
            assert!((stats.variance_population - 1.25).abs() < 1e-12);
            assert!((stats.variance_sample.unwrap() - 5.0 / 3.0).abs() < 1e-12);
            assert_eq!(stats.min, 1.0);
            assert_eq!(stats.max, 4.0);
        }

        #[test]
        fn l0_summary_rejects_overflowed_mean_sum() {
            match summary_stats(&[f64::MAX, f64::MAX]) {
                Err(SummaryError::NonFiniteResult { operation, value }) => {
                    assert_eq!(operation, "summary mean sum");
                    assert!(value.is_infinite());
                }
                other => panic!("expected summary overflow error, got {other:?}"),
            }
        }

        #[test]
        fn l0_summary_rejects_overflowed_variance_sum() {
            match summary_stats(&[f64::MAX, -f64::MAX]) {
                Err(SummaryError::NonFiniteResult { operation, value }) => {
                    assert_eq!(operation, "summary variance sum");
                    assert!(value.is_infinite());
                }
                other => panic!("expected summary variance overflow error, got {other:?}"),
            }
        }

        #[test]
        fn l0_weighted_summary_stats_match_hand_computed_values() {
            let values = [0.0, 10.0, 20.0];
            let weights = [1.0, 2.0, 1.0];

            let stats = weighted_summary_stats(&values, &weights).unwrap();

            assert_eq!(stats.count, 3);
            assert_eq!(stats.total_weight, 4.0);
            assert_eq!(stats.mean, 10.0);
            assert!((stats.variance_population - 50.0).abs() < 1e-12);
            assert!((stats.std_dev_population - 50.0_f64.sqrt()).abs() < 1e-12);
            assert_eq!(stats.min, 0.0);
            assert_eq!(stats.max, 20.0);
        }

        #[test]
        fn l0_weighted_summary_rejects_overflowed_weight_sum() {
            match weighted_summary_stats(&[1.0, 1.0], &[f64::MAX, f64::MAX]) {
                Err(SummaryError::NonFiniteResult { operation, value }) => {
                    assert_eq!(operation, "weighted total weight");
                    assert!(value.is_infinite());
                }
                other => panic!("expected weighted summary overflow error, got {other:?}"),
            }
        }

        #[test]
        fn l0_median_handles_even_and_odd_counts() {
            assert_eq!(median(&[3.0, 1.0, 2.0]).unwrap(), 2.0);
            assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]).unwrap(), 2.5);
        }

        #[test]
        fn l0_quantile_uses_r7_interpolation() {
            let values = [0.0, 10.0, 20.0, 30.0, 40.0];
            assert_eq!(quantile_sorted_copy(&values, 0.25).unwrap(), 10.0);
            assert_eq!(quantile_sorted_copy(&values, 0.125).unwrap(), 5.0);
        }

        #[test]
        fn l0_rejects_empty_and_non_finite_samples() {
            assert_eq!(mean(&[]), Err(SummaryError::EmptySample));
            assert!(matches!(
                mean(&[1.0, f64::NAN]),
                Err(SummaryError::NonFiniteValue { index: 1, .. })
            ));
        }

        #[test]
        fn l0_rejects_invalid_quantile() {
            assert_eq!(
                quantile_sorted_copy(&[1.0], 1.5),
                Err(SummaryError::InvalidQuantile(1.5))
            );
        }

        #[test]
        fn l0_quantile_rejects_overflowed_interpolation_span() {
            match quantile_sorted_copy(&[-f64::MAX, f64::MAX], 0.5) {
                Err(SummaryError::NonFiniteResult { operation, value }) => {
                    assert_eq!(operation, "quantile span");
                    assert!(value.is_infinite());
                }
                other => panic!("expected quantile overflow error, got {other:?}"),
            }
        }

        #[test]
        fn l0_rejects_reversed_percentile_interval_quantiles() {
            assert_eq!(
                percentile_interval_sorted_copy(&[1.0, 2.0, 3.0], 0.90, 0.10),
                Err(SummaryError::InvalidIntervalQuantiles {
                    low: 0.90,
                    high: 0.10
                })
            );
        }

        #[test]
        fn l0_rejects_invalid_weighted_inputs() {
            assert_eq!(
                weighted_mean(&[1.0, 2.0], &[1.0]),
                Err(SummaryError::LengthMismatch {
                    values: 2,
                    weights: 1
                })
            );
            assert_eq!(
                weighted_mean(&[1.0], &[-1.0]),
                Err(SummaryError::InvalidWeight {
                    index: 0,
                    value: -1.0
                })
            );
            assert_eq!(
                weighted_mean(&[1.0], &[0.0]),
                Err(SummaryError::ZeroTotalWeight)
            );
        }
    }
}

pub mod scoring {
    use crate::summary::{median, summary_stats, SummaryError};
    use thiserror::Error;

    #[derive(Debug, Error, Clone, PartialEq)]
    pub enum ScoringError {
        #[error(transparent)]
        Summary(#[from] SummaryError),
        #[error("[INPUT] value and weight lengths differ: {values} values vs {weights} weights")]
        LengthMismatch { values: usize, weights: usize },
        #[error("[INPUT] weight contains negative or non-finite value {value} at index {index}")]
        InvalidWeight { index: usize, value: f64 },
        #[error("[INPUT] total weight must be positive")]
        ZeroTotalWeight,
        #[error("[INPUT] age must be non-negative and finite, got {0}")]
        InvalidAge(f64),
        #[error("[INPUT] half-life must be positive and finite, got {0}")]
        InvalidHalfLife(f64),
        #[error("[INPUT] clip lower bound {low} exceeds upper bound {high}")]
        InvalidClipBounds { low: f64, high: f64 },
        #[error("[NUMERIC] {operation} produced non-finite value {value}")]
        NonFiniteResult { operation: &'static str, value: f64 },
    }

    pub fn min_max_scores(values: &[f64]) -> Result<Vec<f64>, ScoringError> {
        let stats = summary_stats(values)?;
        if stats.min == stats.max {
            return Ok(vec![0.5; values.len()]);
        }
        let span = stats.max - stats.min;
        validate_result("min-max span", span)?;
        values
            .iter()
            .map(|value| {
                let score = (value - stats.min) / span;
                validate_result("min-max score", score)?;
                Ok(score)
            })
            .collect()
    }

    pub fn z_scores(values: &[f64]) -> Result<Vec<f64>, ScoringError> {
        let stats = summary_stats(values)?;
        if stats.std_dev_population == 0.0 {
            return Ok(vec![0.0; values.len()]);
        }
        values
            .iter()
            .map(|value| {
                let score = (value - stats.mean) / stats.std_dev_population;
                validate_result("z-score", score)?;
                Ok(score)
            })
            .collect()
    }

    pub fn robust_z_scores(values: &[f64]) -> Result<Vec<f64>, ScoringError> {
        let center = median(values)?;
        let deviations: Vec<f64> = values.iter().map(|value| (value - center).abs()).collect();
        let mad = median(&deviations)?;
        if mad == 0.0 {
            return Ok(vec![0.0; values.len()]);
        }
        values
            .iter()
            .map(|value| {
                let score = 0.674_489_75 * (value - center) / mad;
                validate_result("robust z-score", score)?;
                Ok(score)
            })
            .collect()
    }

    pub fn percentile_ranks(values: &[f64]) -> Result<Vec<f64>, ScoringError> {
        summary_stats(values)?;
        if values.len() == 1 {
            return Ok(vec![0.5]);
        }

        let mut sorted: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
        sorted.sort_by(|(left_index, left), (right_index, right)| {
            left.total_cmp(right)
                .then_with(|| left_index.cmp(right_index))
        });

        let mut ranks = vec![0.0; values.len()];
        let mut start = 0usize;
        while start < sorted.len() {
            let mut end = start + 1;
            while end < sorted.len() && sorted[end].1 == sorted[start].1 {
                end += 1;
            }
            let average_rank = (start + end - 1) as f64 / 2.0;
            let percentile = average_rank / (values.len() - 1) as f64;
            validate_result("percentile rank", percentile)?;
            for &(index, _) in &sorted[start..end] {
                ranks[index] = percentile;
            }
            start = end;
        }
        Ok(ranks)
    }

    pub fn clipped_unit_score(value: f64, low: f64, high: f64) -> Result<f64, ScoringError> {
        if !low.is_finite() || !high.is_finite() {
            return Err(ScoringError::NonFiniteResult {
                operation: "clip bound",
                value: if !low.is_finite() { low } else { high },
            });
        }
        if low > high {
            return Err(ScoringError::InvalidClipBounds { low, high });
        }
        if !value.is_finite() {
            return Err(ScoringError::NonFiniteResult {
                operation: "clip value",
                value,
            });
        }
        Ok(value.clamp(low, high))
    }

    pub fn weighted_component_score(values: &[f64], weights: &[f64]) -> Result<f64, ScoringError> {
        if values.len() != weights.len() {
            return Err(ScoringError::LengthMismatch {
                values: values.len(),
                weights: weights.len(),
            });
        }
        summary_stats(values)?;
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        for (index, (&value, &weight)) in values.iter().zip(weights).enumerate() {
            if !weight.is_finite() || weight < 0.0 {
                return Err(ScoringError::InvalidWeight {
                    index,
                    value: weight,
                });
            }
            weighted_sum += value * weight;
            total_weight += weight;
            validate_result("weighted component sum", weighted_sum)?;
            validate_result("weighted component total weight", total_weight)?;
        }
        if total_weight == 0.0 {
            return Err(ScoringError::ZeroTotalWeight);
        }
        let score = weighted_sum / total_weight;
        validate_result("weighted component score", score)?;
        Ok(score)
    }

    pub fn exponential_recency_score(
        age_seconds: f64,
        half_life_seconds: f64,
    ) -> Result<f64, ScoringError> {
        if !age_seconds.is_finite() || age_seconds < 0.0 {
            return Err(ScoringError::InvalidAge(age_seconds));
        }
        if !half_life_seconds.is_finite() || half_life_seconds <= 0.0 {
            return Err(ScoringError::InvalidHalfLife(half_life_seconds));
        }
        let score = 2.0_f64.powf(-age_seconds / half_life_seconds);
        validate_result("exponential recency score", score)?;
        Ok(score)
    }

    fn validate_result(operation: &'static str, value: f64) -> Result<(), ScoringError> {
        if value.is_finite() {
            Ok(())
        } else {
            Err(ScoringError::NonFiniteResult { operation, value })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn l0_min_max_scores_are_unit_scaled() {
            assert_eq!(
                min_max_scores(&[10.0, 20.0, 30.0]).unwrap(),
                vec![0.0, 0.5, 1.0]
            );
            assert_eq!(min_max_scores(&[5.0, 5.0]).unwrap(), vec![0.5, 0.5]);
        }

        #[test]
        fn l0_z_scores_are_centered_and_handle_zero_variance() {
            let scores = z_scores(&[1.0, 2.0, 3.0]).unwrap();
            assert!((scores[0] + 1.224_744_871_391_589).abs() < 1e-12);
            assert_eq!(scores[1], 0.0);
            assert!((scores[2] - 1.224_744_871_391_589).abs() < 1e-12);
            assert_eq!(z_scores(&[4.0, 4.0]).unwrap(), vec![0.0, 0.0]);
        }

        #[test]
        fn l0_robust_z_scores_use_median_absolute_deviation() {
            let scores = robust_z_scores(&[1.0, 2.0, 100.0]).unwrap();
            assert!((scores[0] + 0.674_489_75).abs() < 1e-12);
            assert_eq!(scores[1], 0.0);
            assert!((scores[2] - 66.099_995_5).abs() < 1e-9);
            assert_eq!(robust_z_scores(&[7.0, 7.0]).unwrap(), vec![0.0, 0.0]);
        }

        #[test]
        fn l0_percentile_ranks_are_deterministic_with_ties() {
            assert_eq!(
                percentile_ranks(&[30.0, 10.0, 10.0, 20.0]).unwrap(),
                vec![1.0, 1.0 / 6.0, 1.0 / 6.0, 2.0 / 3.0]
            );
            assert_eq!(percentile_ranks(&[42.0]).unwrap(), vec![0.5]);
        }

        #[test]
        fn l0_weighted_component_score_aggregates_explainable_components() {
            assert_eq!(
                weighted_component_score(&[0.5, 1.0, 0.0], &[1.0, 2.0, 1.0]).unwrap(),
                0.625
            );
            assert_eq!(
                weighted_component_score(&[1.0], &[0.0]),
                Err(ScoringError::ZeroTotalWeight)
            );
        }

        #[test]
        fn l0_exponential_recency_score_uses_half_life() {
            assert_eq!(exponential_recency_score(0.0, 10.0).unwrap(), 1.0);
            assert_eq!(exponential_recency_score(10.0, 10.0).unwrap(), 0.5);
            assert_eq!(
                exponential_recency_score(-1.0, 10.0),
                Err(ScoringError::InvalidAge(-1.0))
            );
        }

        #[test]
        fn l0_clipped_unit_score_validates_bounds() {
            assert_eq!(clipped_unit_score(1.5, 0.0, 1.0).unwrap(), 1.0);
            assert_eq!(clipped_unit_score(-0.5, 0.0, 1.0).unwrap(), 0.0);
            assert_eq!(
                clipped_unit_score(0.5, 1.0, 0.0),
                Err(ScoringError::InvalidClipBounds {
                    low: 1.0,
                    high: 0.0
                })
            );
        }

        #[test]
        fn l0_scoring_rejects_non_finite_samples() {
            assert!(matches!(
                min_max_scores(&[1.0, f64::NAN]),
                Err(ScoringError::Summary(SummaryError::NonFiniteValue {
                    index: 1,
                    ..
                }))
            ));
        }
    }
}

pub mod resampling {
    use crate::summary::{percentile_interval_sorted_copy, SummaryError};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};
    use thiserror::Error;

    #[derive(Debug, Error, Clone, PartialEq)]
    pub enum BootstrapError {
        #[error("[INPUT] empty bootstrap sample")]
        EmptySample,
        #[error("[INPUT] bootstrap replicate count must be positive")]
        ZeroReplicates,
        #[error("[INPUT] bootstrap statistic for replicate {replicate} is non-finite: {value}")]
        NonFiniteStatistic { replicate: usize, value: f64 },
        #[error(transparent)]
        Summary(#[from] SummaryError),
    }

    pub fn bootstrap_statistics<T, F>(
        sample: &[T],
        n_replicates: usize,
        seed: u64,
        statistic: F,
    ) -> Result<Vec<f64>, BootstrapError>
    where
        T: Clone,
        F: Fn(&[T]) -> f64,
    {
        if sample.is_empty() {
            return Err(BootstrapError::EmptySample);
        }
        if n_replicates == 0 {
            return Err(BootstrapError::ZeroReplicates);
        }

        let mut rng = SmallRng::seed_from_u64(seed);
        let mut stats = Vec::with_capacity(n_replicates);
        for replicate in 0..n_replicates {
            let resample: Vec<T> = (0..sample.len())
                .map(|_| sample[rng.gen_range(0..sample.len())].clone())
                .collect();
            let value = statistic(&resample);
            if !value.is_finite() {
                return Err(BootstrapError::NonFiniteStatistic { replicate, value });
            }
            stats.push(value);
        }
        Ok(stats)
    }

    pub fn bootstrap_percentile_interval<T, F>(
        sample: &[T],
        n_replicates: usize,
        seed: u64,
        statistic: F,
        low_q: f64,
        high_q: f64,
    ) -> Result<(f64, f64), BootstrapError>
    where
        T: Clone,
        F: Fn(&[T]) -> f64,
    {
        let stats = bootstrap_statistics(sample, n_replicates, seed, statistic)?;
        Ok(percentile_interval_sorted_copy(&stats, low_q, high_q)?)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn l0_bootstrap_statistics_are_seed_reproducible() {
            let sample = [1.0, 2.0, 3.0, 4.0];
            let stat = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;

            let a = bootstrap_statistics(&sample, 20, 42, stat).unwrap();
            let b = bootstrap_statistics(&sample, 20, 42, stat).unwrap();
            let c = bootstrap_statistics(&sample, 20, 43, stat).unwrap();

            assert_eq!(a, b);
            assert_ne!(a, c);
        }

        #[test]
        fn l0_bootstrap_percentile_interval_is_ordered() {
            let sample = [1.0, 2.0, 3.0, 4.0, 5.0];
            let stat = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;

            let (lo, hi) = bootstrap_percentile_interval(&sample, 200, 7, stat, 0.025, 0.975)
                .expect("bootstrap CI should compute");

            assert!(lo <= hi);
            assert!((1.0..=5.0).contains(&lo));
            assert!((1.0..=5.0).contains(&hi));
        }

        #[test]
        fn l0_bootstrap_rejects_empty_and_zero_replicates() {
            let stat = |xs: &[f64]| xs.iter().sum::<f64>();
            assert_eq!(
                bootstrap_statistics::<f64, _>(&[], 10, 1, stat),
                Err(BootstrapError::EmptySample)
            );
            assert_eq!(
                bootstrap_statistics(&[1.0], 0, 1, stat),
                Err(BootstrapError::ZeroReplicates)
            );
        }

        #[test]
        fn l0_bootstrap_rejects_non_finite_statistics() {
            let err = bootstrap_statistics(&[1.0, 2.0], 1, 1, |_| f64::NAN).unwrap_err();
            assert!(matches!(
                err,
                BootstrapError::NonFiniteStatistic {
                    replicate: 0,
                    value
                } if value.is_nan()
            ));
        }

        #[test]
        fn l0_bootstrap_rejects_reversed_percentile_interval_quantiles() {
            let stat = |xs: &[f64]| xs.iter().sum::<f64>();

            assert_eq!(
                bootstrap_percentile_interval(&[1.0, 2.0], 10, 1, stat, 0.75, 0.25),
                Err(BootstrapError::Summary(
                    SummaryError::InvalidIntervalQuantiles {
                        low: 0.75,
                        high: 0.25
                    }
                ))
            );
        }
    }
}

pub mod hypothesis {
    use crate::probability::{regularized_incomplete_beta, ProbabilityError};
    use std::collections::{HashMap, HashSet};
    use thiserror::Error;

    #[derive(Debug, Error, Clone, PartialEq)]
    pub enum HypothesisError {
        #[error("[INPUT] empty reference sample")]
        EmptySample,
        #[error("[INPUT] non-finite statistic {value} at index {index}")]
        NonFiniteStatistic { index: usize, value: f64 },
        #[error("[INPUT] probability must be in [0, 1], got {0}")]
        InvalidProbability(f64),
        #[error("[INPUT] ESS must be positive and finite, got {0}")]
        InvalidEss(f64),
        #[error("[INPUT] duplicate hypothesis test name '{0}'")]
        DuplicateTestName(String),
        #[error(transparent)]
        Probability(#[from] ProbabilityError),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Tail {
        Lower,
        Upper,
        TwoSidedDistanceFromCenter,
    }

    pub fn empirical_p_value(
        observed: f64,
        reference: &[f64],
        tail: Tail,
    ) -> Result<(usize, usize, f64), HypothesisError> {
        if reference.is_empty() {
            return Err(HypothesisError::EmptySample);
        }
        validate_finite(usize::MAX, observed)?;
        for (index, &value) in reference.iter().enumerate() {
            validate_finite(index, value)?;
        }

        let count = match tail {
            Tail::Lower => reference.iter().filter(|&&value| value <= observed).count(),
            Tail::Upper => reference.iter().filter(|&&value| value >= observed).count(),
            Tail::TwoSidedDistanceFromCenter => {
                let observed_distance = (observed - 0.5).abs();
                reference
                    .iter()
                    .filter(|&&value| (value - 0.5).abs() >= observed_distance)
                    .count()
            }
        };
        let total = reference.len();
        Ok((count, total, count as f64 / total as f64))
    }

    pub fn ess_beta_median(p_raw: f64, ess: f64) -> Result<f64, HypothesisError> {
        validate_probability(p_raw)?;
        validate_ess(ess)?;
        let a = p_raw * ess + 1.0;
        let b = (1.0 - p_raw) * ess + 1.0;
        Ok(((a - 1.0 / 3.0) / (a + b - 2.0 / 3.0)).clamp(0.0, 1.0))
    }

    pub fn bayesian_detection_score(
        threshold: f64,
        p_raw: f64,
        ess: f64,
    ) -> Result<f64, HypothesisError> {
        validate_probability(threshold)?;
        validate_probability(p_raw)?;
        validate_ess(ess)?;
        let a = p_raw * ess + 1.0;
        let b = (1.0 - p_raw) * ess + 1.0;
        Ok(regularized_incomplete_beta(threshold, a, b)?.clamp(0.0, 1.0))
    }

    pub fn holm_bonferroni(p_values: &[f64]) -> Result<Vec<f64>, HypothesisError> {
        let m = p_values.len();
        if m == 0 {
            return Ok(Vec::new());
        }
        validate_probabilities(p_values)?;
        let mut indexed: Vec<(usize, f64)> = p_values.iter().copied().enumerate().collect();
        indexed.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        let mut corrected = vec![0.0; m];
        let mut running_max = 0.0_f64;
        for (rank, (original_index, p)) in indexed.into_iter().enumerate() {
            let adjusted = ((m - rank) as f64 * p).clamp(0.0, 1.0);
            running_max = running_max.max(adjusted);
            corrected[original_index] = running_max;
        }
        Ok(corrected)
    }

    pub fn holm_bonferroni_named(
        p_values: &[(String, f64)],
    ) -> Result<HashMap<String, f64>, HypothesisError> {
        let mut names = HashSet::with_capacity(p_values.len());
        for (name, _) in p_values {
            if !names.insert(name.as_str()) {
                return Err(HypothesisError::DuplicateTestName(name.clone()));
            }
        }
        let raw: Vec<f64> = p_values.iter().map(|(_, p)| *p).collect();
        let corrected = holm_bonferroni(&raw)?;
        Ok(p_values
            .iter()
            .zip(corrected)
            .map(|((name, _), p)| (name.clone(), p))
            .collect())
    }

    pub fn benjamini_hochberg(p_values: &[f64]) -> Result<Vec<f64>, HypothesisError> {
        let m = p_values.len();
        if m == 0 {
            return Ok(Vec::new());
        }
        validate_probabilities(p_values)?;
        let mut indexed: Vec<(usize, f64)> = p_values.iter().copied().enumerate().collect();
        indexed.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        let mut corrected = vec![0.0; m];
        let mut running_min = 1.0_f64;
        for (reverse_rank, (original_index, p)) in indexed.into_iter().rev().enumerate() {
            let rank = m - reverse_rank;
            let adjusted = (p * m as f64 / rank as f64).clamp(0.0, 1.0);
            running_min = running_min.min(adjusted);
            corrected[original_index] = running_min;
        }
        Ok(corrected)
    }

    fn validate_probabilities(p_values: &[f64]) -> Result<(), HypothesisError> {
        for (index, &value) in p_values.iter().enumerate() {
            validate_probability_at(index, value)?;
        }
        Ok(())
    }

    fn validate_probability(value: f64) -> Result<(), HypothesisError> {
        validate_probability_at(usize::MAX, value)
    }

    fn validate_probability_at(index: usize, value: f64) -> Result<(), HypothesisError> {
        if !value.is_finite() {
            return Err(HypothesisError::NonFiniteStatistic { index, value });
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(HypothesisError::InvalidProbability(value));
        }
        Ok(())
    }

    fn validate_finite(index: usize, value: f64) -> Result<(), HypothesisError> {
        if !value.is_finite() {
            return Err(HypothesisError::NonFiniteStatistic { index, value });
        }
        Ok(())
    }

    fn validate_ess(ess: f64) -> Result<(), HypothesisError> {
        if !ess.is_finite() || ess <= 0.0 {
            return Err(HypothesisError::InvalidEss(ess));
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn l0_empirical_p_values_cover_all_tails() {
            let reference = [0.0, 0.1, 0.2, 0.8, 0.9];

            assert_eq!(
                empirical_p_value(0.15, &reference, Tail::Lower).unwrap(),
                (2, 5, 0.4)
            );
            assert_eq!(
                empirical_p_value(0.85, &reference, Tail::Upper).unwrap(),
                (1, 5, 0.2)
            );
            assert_eq!(
                empirical_p_value(0.1, &reference, Tail::TwoSidedDistanceFromCenter).unwrap(),
                (3, 5, 0.6)
            );
        }

        #[test]
        fn l0_ess_beta_median_matches_existing_formula() {
            let p = ess_beta_median(0.01, 70.0).unwrap();
            assert!((p - 0.0191588785046729).abs() < 1e-12);
        }

        #[test]
        fn l0_holm_bonferroni_is_step_down_and_dominates_raw() {
            let raw = [0.001, 0.02, 0.03, 0.90];
            let corrected = holm_bonferroni(&raw).unwrap();

            assert_eq!(corrected.len(), raw.len());
            for (p, p_holm) in raw.iter().zip(&corrected) {
                assert!(p_holm + 1e-12 >= *p);
            }
            assert!((corrected[0] - 0.004).abs() < 1e-12);
            assert!((corrected[1] - 0.06).abs() < 1e-12);
            assert!((corrected[2] - 0.06).abs() < 1e-12);
            assert_eq!(corrected[3], 0.90);
        }

        #[test]
        fn l0_holm_bonferroni_named_rejects_duplicate_test_names() {
            let raw = vec![
                ("race::primary".to_string(), 0.01),
                ("race::primary".to_string(), 0.02),
            ];

            assert_eq!(
                holm_bonferroni_named(&raw),
                Err(HypothesisError::DuplicateTestName(
                    "race::primary".to_string()
                ))
            );
        }

        #[test]
        fn l0_benjamini_hochberg_is_bounded() {
            let corrected = benjamini_hochberg(&[0.001, 0.02, 0.03, 0.90]).unwrap();
            assert!(corrected.iter().all(|p| (0.0..=1.0).contains(p)));
            assert!((corrected[0] - 0.004).abs() < 1e-12);
        }

        #[test]
        fn l0_rejects_invalid_probability_and_ess() {
            assert_eq!(
                holm_bonferroni(&[1.2]),
                Err(HypothesisError::InvalidProbability(1.2))
            );
            assert_eq!(
                ess_beta_median(0.5, 0.0),
                Err(HypothesisError::InvalidEss(0.0))
            );
        }
    }
}

pub mod mcmc {
    use thiserror::Error;

    #[derive(Debug, Error, Clone, PartialEq)]
    pub enum DiagnosticsError {
        #[error("[INPUT] requires >=4 parallel chains for Gelman-Rubin R-hat; got {0}")]
        InsufficientChains(usize),
        #[error("[INPUT] empty chain at index {0}")]
        EmptyChain(usize),
        #[error("[INPUT] chains have differing lengths: {0:?}")]
        UnequalChainLengths(Vec<usize>),
        #[error("[INPUT] chain {chain_index} contains non-finite value {value} at sample {sample_index}")]
        NonFiniteChainValue {
            chain_index: usize,
            sample_index: usize,
            value: f64,
        },
        #[error("[INPUT] trace contains non-finite value {value} at index {index}")]
        NonFiniteTraceValue { index: usize, value: f64 },
        #[error("[INPUT] autocorrelation lag value must be in [0, 1], got {value} at lag {lag}")]
        InvalidAutocorrelationValue { lag: usize, value: f64 },
        #[error("[NUMERIC] {operation} produced non-finite value {value}")]
        NonFiniteResult { operation: &'static str, value: f64 },
        #[error("[INPUT] empty partition trajectory")]
        EmptyTrajectory,
        #[error("[INPUT] empty partition at index {0}")]
        EmptyPartition(usize),
        #[error(
            "[INPUT] partitions have differing unit counts: first={first}, at index {idx}={got}"
        )]
        PartitionLengthMismatch {
            first: usize,
            idx: usize,
            got: usize,
        },
    }

    pub fn gelman_rubin_rhat(chains: &[&[f64]]) -> Result<f64, DiagnosticsError> {
        let m = chains.len();
        if m < 4 {
            return Err(DiagnosticsError::InsufficientChains(m));
        }
        let n = chains[0].len();
        if n == 0 {
            return Err(DiagnosticsError::EmptyChain(0));
        }
        for (i, chain) in chains.iter().enumerate() {
            if chain.is_empty() {
                return Err(DiagnosticsError::EmptyChain(i));
            }
            if chain.len() != n {
                return Err(DiagnosticsError::UnequalChainLengths(
                    chains.iter().map(|c| c.len()).collect(),
                ));
            }
            for (sample_index, &value) in chain.iter().enumerate() {
                if !value.is_finite() {
                    return Err(DiagnosticsError::NonFiniteChainValue {
                        chain_index: i,
                        sample_index,
                        value,
                    });
                }
            }
        }

        let mut chain_means = Vec::with_capacity(m);
        for chain in chains {
            let sum = checked_sum(chain.iter().copied(), "rhat chain mean sum")?;
            let mean = sum / n as f64;
            validate_result("rhat chain mean", mean)?;
            chain_means.push(mean);
        }
        let grand_mean =
            checked_sum(chain_means.iter().copied(), "rhat grand mean sum")? / m as f64;
        validate_result("rhat grand mean", grand_mean)?;

        let mut chain_vars = Vec::with_capacity(m);
        for (chain, mean) in chains.iter().zip(&chain_means) {
            let mut sum_sq = 0.0;
            for &x in *chain {
                let centered = x - mean;
                validate_result("rhat centered value", centered)?;
                sum_sq += centered.powi(2);
                validate_result("rhat chain variance sum", sum_sq)?;
            }
            let variance = sum_sq / (n - 1).max(1) as f64;
            validate_result("rhat chain variance", variance)?;
            chain_vars.push(variance);
        }

        let mut between_sum = 0.0;
        for mean in &chain_means {
            let centered = mean - grand_mean;
            validate_result("rhat between-chain centered mean", centered)?;
            between_sum += centered.powi(2);
            validate_result("rhat between-chain variance sum", between_sum)?;
        }
        let b_over_n = between_sum / (m - 1).max(1) as f64;
        validate_result("rhat between-chain variance", b_over_n)?;
        let w =
            checked_sum(chain_vars.iter().copied(), "rhat within-chain variance sum")? / m as f64;
        validate_result("rhat within-chain variance", w)?;
        if w == 0.0 {
            return Ok(1.0);
        }

        let n_f = n as f64;
        let variance_estimate = ((n_f - 1.0) / n_f) * w + b_over_n;
        validate_result("rhat variance estimate", variance_estimate)?;
        let numerator = variance_estimate.sqrt();
        validate_result("rhat numerator", numerator)?;
        let denominator = w.sqrt();
        validate_result("rhat denominator", denominator)?;
        let rhat = numerator / denominator;
        validate_result("rhat", rhat)?;
        Ok(rhat)
    }

    pub fn effective_sample_size(trace: &[f64]) -> Result<f64, DiagnosticsError> {
        let n = trace.len();
        for (index, &value) in trace.iter().enumerate() {
            if !value.is_finite() {
                return Err(DiagnosticsError::NonFiniteTraceValue { index, value });
            }
        }
        if n < 4 {
            return Ok(n as f64);
        }
        let mean = checked_sum(trace.iter().copied(), "ess mean sum")? / n as f64;
        validate_result("ess mean", mean)?;
        let mut centered = Vec::with_capacity(n);
        for &x in trace {
            let value = x - mean;
            validate_result("ess centered value", value)?;
            centered.push(value);
        }
        let var = checked_sum(centered.iter().map(|x| x * x), "ess variance sum")? / n as f64;
        validate_result("ess variance", var)?;
        if var == 0.0 {
            return Ok(n as f64);
        }

        let autocorr_at = |lag: usize| -> Result<f64, DiagnosticsError> {
            if lag >= n {
                return Ok(0.0);
            }
            let mut sum = 0.0;
            for i in 0..(n - lag) {
                sum += centered[i] * centered[i + lag];
                validate_result("ess autocorrelation sum", sum)?;
            }
            let denom = n as f64 * var;
            validate_result("ess autocorrelation denominator", denom)?;
            let rho = sum / denom;
            validate_result("ess autocorrelation", rho)?;
            Ok(rho)
        };

        let mut sum_rho = 0.0_f64;
        let mut prev_pair = f64::INFINITY;
        let max_lag = (n / 4).max(2);
        let mut k = 0usize;
        while 2 * k + 2 <= max_lag {
            let pair = autocorr_at(2 * k + 1)? + autocorr_at(2 * k + 2)?;
            validate_result("ess autocorrelation pair", pair)?;
            if pair <= 0.0 {
                break;
            }
            let pair = pair.min(prev_pair);
            sum_rho += pair;
            validate_result("ess autocorrelation pair sum", sum_rho)?;
            prev_pair = pair;
            k += 1;
        }

        let denom = 1.0 + 2.0 * sum_rho;
        validate_result("ess denominator", denom)?;
        if denom <= 0.0 {
            Ok(n as f64)
        } else {
            let ess = n as f64 / denom;
            validate_result("ess", ess)?;
            Ok(ess)
        }
    }

    pub fn hamming_autocorrelation(
        partitions: &[Vec<usize>],
        max_lag: usize,
    ) -> Result<Vec<f64>, DiagnosticsError> {
        let t = partitions.len();
        if t == 0 {
            return Err(DiagnosticsError::EmptyTrajectory);
        }
        let n_units = partitions[0].len();
        if n_units == 0 {
            return Err(DiagnosticsError::EmptyPartition(0));
        }
        for (idx, partition) in partitions.iter().enumerate() {
            if partition.is_empty() {
                return Err(DiagnosticsError::EmptyPartition(idx));
            }
            if partition.len() != n_units {
                return Err(DiagnosticsError::PartitionLengthMismatch {
                    first: n_units,
                    idx,
                    got: partition.len(),
                });
            }
        }

        let max_lag = max_lag.min(t.saturating_sub(1));
        let mut out = Vec::with_capacity(max_lag + 1);
        out.push(0.0);
        for lag in 1..=max_lag {
            let mut total = 0.0;
            let pairs = t - lag;
            for i in 0..pairs {
                let mut diff = 0usize;
                for unit in 0..n_units {
                    if partitions[i][unit] != partitions[i + lag][unit] {
                        diff += 1;
                    }
                }
                total += diff as f64 / n_units as f64;
            }
            out.push(if pairs > 0 { total / pairs as f64 } else { 0.0 });
        }
        Ok(out)
    }

    pub fn integrated_autocorrelation_time(
        autocorr_per_lag: &[f64],
    ) -> Result<f64, DiagnosticsError> {
        for (lag, &value) in autocorr_per_lag.iter().enumerate() {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(DiagnosticsError::InvalidAutocorrelationValue { lag, value });
            }
        }
        if autocorr_per_lag.len() <= 1 {
            return Ok(1.0);
        }
        let mut tau = 1.0;
        for &h in &autocorr_per_lag[1..] {
            let rho = 1.0 - h;
            if rho <= 0.0 {
                break;
            }
            tau += 2.0 * rho;
        }
        Ok(tau)
    }

    fn checked_sum<I>(values: I, operation: &'static str) -> Result<f64, DiagnosticsError>
    where
        I: IntoIterator<Item = f64>,
    {
        let mut sum = 0.0;
        for value in values {
            sum += value;
            validate_result(operation, sum)?;
        }
        Ok(sum)
    }

    fn validate_result(operation: &'static str, value: f64) -> Result<(), DiagnosticsError> {
        if !value.is_finite() {
            return Err(DiagnosticsError::NonFiniteResult { operation, value });
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rhat_rejects_too_few_chains() {
            let chain = vec![1.0; 10];
            let chains = vec![chain.as_slice(), chain.as_slice(), chain.as_slice()];
            assert_eq!(
                gelman_rubin_rhat(&chains),
                Err(DiagnosticsError::InsufficientChains(3))
            );
        }

        #[test]
        fn rhat_identical_chains_is_one() {
            let chain = vec![5.0; 50];
            let chains: Vec<&[f64]> = (0..4).map(|_| chain.as_slice()).collect();
            assert!((gelman_rubin_rhat(&chains).unwrap() - 1.0).abs() < 1e-9);
        }

        #[test]
        fn rhat_rejects_non_finite_chain_values() {
            let c1 = vec![1.0, 1.1, 1.2, 1.3];
            let c2 = vec![1.0, f64::NAN, 1.2, 1.3];
            let c3 = vec![1.0, 1.1, 1.2, 1.3];
            let c4 = vec![1.0, 1.1, 1.2, 1.3];
            let chains = vec![c1.as_slice(), c2.as_slice(), c3.as_slice(), c4.as_slice()];

            match gelman_rubin_rhat(&chains) {
                Err(DiagnosticsError::NonFiniteChainValue {
                    chain_index,
                    sample_index,
                    value,
                }) => {
                    assert_eq!((chain_index, sample_index), (1, 1));
                    assert!(value.is_nan());
                }
                other => panic!("expected non-finite chain value error, got {other:?}"),
            }
        }

        #[test]
        fn rhat_rejects_overflowed_variance_aggregate() {
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
        }

        #[test]
        fn ess_constant_trace_returns_n() {
            assert_eq!(effective_sample_size(&vec![5.0; 100]).unwrap(), 100.0);
        }

        #[test]
        fn ess_rejects_non_finite_trace_values() {
            match effective_sample_size(&[1.0, f64::INFINITY, 2.0, 3.0]) {
                Err(DiagnosticsError::NonFiniteTraceValue { index, value }) => {
                    assert_eq!(index, 1);
                    assert!(value.is_infinite());
                }
                other => panic!("expected non-finite trace value error, got {other:?}"),
            }
        }

        #[test]
        fn ess_rejects_overflowed_variance_aggregate() {
            match effective_sample_size(&[f64::MAX, -f64::MAX, f64::MAX, -f64::MAX]) {
                Err(DiagnosticsError::NonFiniteResult { operation, value }) => {
                    assert_eq!(operation, "ess variance sum");
                    assert!(value.is_infinite());
                }
                other => panic!("expected ess overflow error, got {other:?}"),
            }
        }

        #[test]
        fn hamming_rejects_length_mismatch() {
            let partitions = vec![vec![1, 2, 3], vec![1, 2]];
            assert!(matches!(
                hamming_autocorrelation(&partitions, 1),
                Err(DiagnosticsError::PartitionLengthMismatch { .. })
            ));
        }

        #[test]
        fn hamming_rejects_empty_partition() {
            let partitions = vec![Vec::new(), Vec::new()];
            assert_eq!(
                hamming_autocorrelation(&partitions, 1),
                Err(DiagnosticsError::EmptyPartition(0))
            );
        }

        #[test]
        fn tau_lag_zero_only_is_one() {
            assert_eq!(integrated_autocorrelation_time(&[0.0]).unwrap(), 1.0);
        }

        #[test]
        fn tau_rejects_invalid_lag_values() {
            match integrated_autocorrelation_time(&[0.0, f64::NAN]) {
                Err(DiagnosticsError::InvalidAutocorrelationValue { lag, value }) => {
                    assert_eq!(lag, 1);
                    assert!(value.is_nan());
                }
                other => panic!("expected invalid autocorrelation value error, got {other:?}"),
            }
            assert_eq!(
                integrated_autocorrelation_time(&[0.0, 1.2]),
                Err(DiagnosticsError::InvalidAutocorrelationValue { lag: 1, value: 1.2 })
            );
        }
    }
}

pub mod probability {
    use thiserror::Error;

    #[derive(Debug, Error, Clone, PartialEq)]
    pub enum ProbabilityError {
        #[error("[INPUT] normal CDF z-score must be finite, got {0}")]
        NonFiniteZ(f64),
        #[error("[INPUT] beta CDF x must be finite, got {0}")]
        NonFiniteX(f64),
        #[error("[INPUT] beta shape parameter '{name}' must be positive and finite, got {value}")]
        InvalidShape { name: &'static str, value: f64 },
        #[error("[NUMERIC] {operation} produced non-finite value {value}")]
        NonFiniteResult { operation: &'static str, value: f64 },
    }

    /// Standard Normal CDF via Abramowitz & Stegun 7.1.26 approximation.
    pub fn standard_normal_cdf(x: f64) -> Result<f64, ProbabilityError> {
        if !x.is_finite() {
            return Err(ProbabilityError::NonFiniteZ(x));
        }
        let t = x / std::f64::consts::SQRT_2;
        Ok(0.5 * (1.0 + erf_approx(t)))
    }

    pub fn regularized_incomplete_beta(x: f64, a: f64, b: f64) -> Result<f64, ProbabilityError> {
        validate_beta_inputs(x, a, b)?;
        let value = regularized_incomplete_beta_inner(x, a, b)?;
        validate_result("beta cdf", value)?;
        Ok(value)
    }

    fn regularized_incomplete_beta_inner(x: f64, a: f64, b: f64) -> Result<f64, ProbabilityError> {
        if x <= 0.0 {
            return Ok(0.0);
        }
        if x >= 1.0 {
            return Ok(1.0);
        }

        let shape_window = a + b + 2.0;
        validate_result("beta shape window", shape_window)?;
        let branch_threshold = (a + 1.0) / shape_window;
        validate_result("beta branch threshold", branch_threshold)?;
        if x > branch_threshold {
            let complement = 1.0 - regularized_incomplete_beta_inner(1.0 - x, b, a)?;
            validate_result("beta complement", complement)?;
            return Ok(complement);
        }

        let lgamma_a = lgamma(a);
        validate_result("beta lgamma(a)", lgamma_a)?;
        let lgamma_b = lgamma(b);
        validate_result("beta lgamma(b)", lgamma_b)?;
        let lgamma_sum = lgamma(a + b);
        validate_result("beta lgamma(a+b)", lgamma_sum)?;
        let lbeta = lgamma_a + lgamma_b - lgamma_sum;
        validate_result("beta log normalizer", lbeta)?;
        let exponent = a * x.ln() + b * (1.0 - x).ln() - lbeta;
        validate_result("beta front exponent", exponent)?;
        let front = exponent.exp() / a;
        validate_result("beta front factor", front)?;
        let fraction = beta_continued_fraction(x, a, b)?;
        let value = front * fraction;
        validate_result("beta continued fraction product", value)?;
        Ok(value)
    }

    fn validate_beta_inputs(x: f64, a: f64, b: f64) -> Result<(), ProbabilityError> {
        if !x.is_finite() {
            return Err(ProbabilityError::NonFiniteX(x));
        }
        if !a.is_finite() || a <= 0.0 {
            return Err(ProbabilityError::InvalidShape {
                name: "a",
                value: a,
            });
        }
        if !b.is_finite() || b <= 0.0 {
            return Err(ProbabilityError::InvalidShape {
                name: "b",
                value: b,
            });
        }
        Ok(())
    }

    fn beta_continued_fraction(x: f64, a: f64, b: f64) -> Result<f64, ProbabilityError> {
        let max_iter = 200;
        let eps = 1e-10;

        let mut c = 1.0;
        let mut d = 1.0 - (a + b) * x / (a + 1.0);
        validate_result("beta continued fraction initial d", d)?;
        if d.abs() < f64::MIN_POSITIVE {
            d = f64::MIN_POSITIVE;
        }
        d = 1.0 / d;
        validate_result("beta continued fraction reciprocal", d)?;
        let mut result = d;

        for m in 1..=max_iter {
            let m = m as f64;

            let dm = m * (b - m) * x / ((a + 2.0 * m - 1.0) * (a + 2.0 * m));
            validate_result("beta continued fraction even term", dm)?;
            d = 1.0 + dm * d;
            validate_result("beta continued fraction even d", d)?;
            if d.abs() < f64::MIN_POSITIVE {
                d = f64::MIN_POSITIVE;
            }
            c = 1.0 + dm / c;
            validate_result("beta continued fraction even c", c)?;
            if c.abs() < f64::MIN_POSITIVE {
                c = f64::MIN_POSITIVE;
            }
            d = 1.0 / d;
            validate_result("beta continued fraction even reciprocal", d)?;
            result *= d * c;
            validate_result("beta continued fraction result", result)?;

            let dm = -(a + m) * (a + b + m) * x / ((a + 2.0 * m) * (a + 2.0 * m + 1.0));
            validate_result("beta continued fraction odd term", dm)?;
            d = 1.0 + dm * d;
            validate_result("beta continued fraction odd d", d)?;
            if d.abs() < f64::MIN_POSITIVE {
                d = f64::MIN_POSITIVE;
            }
            c = 1.0 + dm / c;
            validate_result("beta continued fraction odd c", c)?;
            if c.abs() < f64::MIN_POSITIVE {
                c = f64::MIN_POSITIVE;
            }
            d = 1.0 / d;
            validate_result("beta continued fraction odd reciprocal", d)?;
            let delta = d * c;
            validate_result("beta continued fraction delta", delta)?;
            result *= delta;
            validate_result("beta continued fraction result", result)?;

            if (delta - 1.0).abs() < eps {
                break;
            }
        }
        Ok(result)
    }

    fn erf_approx(x: f64) -> f64 {
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;
        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
        sign * y
    }

    fn lgamma(z: f64) -> f64 {
        let g = 7.0_f64;
        let c = [
            0.9999999999998099,
            676.5203681218851,
            -1259.1392167224028,
            771.3234287776531,
            -176.6150291621406,
            12.507343278686905,
            -0.13857109526572012,
            9.984369578019572e-6,
            1.5056327351493116e-7,
        ];
        if z < 0.5 {
            std::f64::consts::PI.ln() - (std::f64::consts::PI * z).sin().ln() - lgamma(1.0 - z)
        } else {
            let z = z - 1.0;
            let mut x = c[0];
            for (i, &ci) in c[1..].iter().enumerate() {
                x += ci / (z + i as f64 + 1.0);
            }
            let t = z + g + 0.5;
            0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + x.ln()
        }
    }

    fn validate_result(operation: &'static str, value: f64) -> Result<(), ProbabilityError> {
        if !value.is_finite() {
            return Err(ProbabilityError::NonFiniteResult { operation, value });
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn beta_boundaries_are_exact() {
            assert!((regularized_incomplete_beta(0.0, 2.0, 3.0).unwrap() - 0.0).abs() < 1e-10);
            assert!((regularized_incomplete_beta(1.0, 2.0, 3.0).unwrap() - 1.0).abs() < 1e-10);
        }

        #[test]
        fn beta_symmetric_midpoint_is_half() {
            assert!((regularized_incomplete_beta(0.5, 2.0, 2.0).unwrap() - 0.5).abs() < 0.01);
        }

        #[test]
        fn beta_rejects_invalid_inputs() {
            match regularized_incomplete_beta(f64::NAN, 2.0, 2.0) {
                Err(ProbabilityError::NonFiniteX(value)) => assert!(value.is_nan()),
                other => panic!("expected non-finite x error, got {other:?}"),
            }
            assert_eq!(
                regularized_incomplete_beta(0.5, 0.0, 2.0),
                Err(ProbabilityError::InvalidShape {
                    name: "a",
                    value: 0.0
                })
            );
            assert_eq!(
                regularized_incomplete_beta(0.5, 2.0, f64::INFINITY),
                Err(ProbabilityError::InvalidShape {
                    name: "b",
                    value: f64::INFINITY
                })
            );
        }

        #[test]
        fn beta_rejects_overflowed_shape_window_before_recursing() {
            match regularized_incomplete_beta(0.5, f64::MAX, f64::MAX) {
                Err(ProbabilityError::NonFiniteResult { operation, value }) => {
                    assert_eq!(operation, "beta shape window");
                    assert!(value.is_infinite());
                }
                other => panic!("expected beta shape overflow error, got {other:?}"),
            }
        }

        #[test]
        fn normal_cdf_matches_known_quantiles() {
            assert!((standard_normal_cdf(0.0).unwrap() - 0.5).abs() < 1e-7);
            assert!((standard_normal_cdf(1.96).unwrap() - 0.975002).abs() < 2e-6);
            assert!((standard_normal_cdf(-1.96).unwrap() - 0.024998).abs() < 2e-6);
        }

        #[test]
        fn normal_cdf_rejects_non_finite_z_scores() {
            assert_eq!(
                standard_normal_cdf(f64::INFINITY),
                Err(ProbabilityError::NonFiniteZ(f64::INFINITY))
            );
            match standard_normal_cdf(f64::NAN) {
                Err(ProbabilityError::NonFiniteZ(value)) => assert!(value.is_nan()),
                other => panic!("expected NonFiniteZ(NaN), got {other:?}"),
            }
        }
    }
}
