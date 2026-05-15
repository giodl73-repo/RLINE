use sha2::{Digest, Sha256};
use thiserror::Error;

pub trait ObjectiveVector {
    fn objective_count(&self) -> usize;
    fn objective_value(&self, index: usize) -> f64;
}

impl<const N: usize> ObjectiveVector for [f64; N] {
    fn objective_count(&self) -> usize {
        N
    }

    fn objective_value(&self, index: usize) -> f64 {
        self[index]
    }
}

impl ObjectiveVector for Vec<f64> {
    fn objective_count(&self) -> usize {
        self.len()
    }

    fn objective_value(&self, index: usize) -> f64 {
        self[index]
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum RoptError {
    #[error("[INPUT] objective vector {point_index} must not be empty")]
    EmptyObjectiveVector { point_index: usize },
    #[error(
        "[INPUT] objective vector {point_index} dimension mismatch: expected {expected}, found {found}"
    )]
    ObjectiveDimensionMismatch {
        point_index: usize,
        expected: usize,
        found: usize,
    },
    #[error("[INPUT] objective vector {point_index} has non-finite value {value} at dimension {dimension}")]
    NonFiniteObjective {
        point_index: usize,
        dimension: usize,
        value: f64,
    },
    #[error("[INPUT] front entry {front_index} references objective index {point_index}, but len is {len}")]
    FrontIndexOutOfBounds {
        front_index: usize,
        point_index: usize,
        len: usize,
    },
    #[error("[INPUT] front entry {front_index} duplicates objective index {point_index}")]
    DuplicateFrontIndex {
        front_index: usize,
        point_index: usize,
    },
    #[error("[NUMERIC] {operation} produced non-finite value {value}")]
    NonFiniteResult { operation: &'static str, value: f64 },
    #[error("[INPUT] seed domain must not be empty")]
    EmptySeedDomain,
    #[error("[INPUT] budgeted selection items must not be empty")]
    EmptyBudgetItems,
    #[error("[INPUT] budget must be positive")]
    NonPositiveBudget,
    #[error("[INPUT] budget item {item_index} must have positive cost")]
    NonPositiveBudgetCost { item_index: usize },
    #[error("[INPUT] budget item {item_index} has invalid score {score}")]
    InvalidBudgetScore { item_index: usize, score: f64 },
    #[error("[INPUT] no budget item fits budget {budget}; minimum item cost is {min_cost}")]
    ImpossibleBudget { budget: usize, min_cost: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedPart {
    U32(u32),
    U64(u64),
    Usize(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BudgetItem {
    pub score: f64,
    pub cost: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetSelection {
    pub selected_indices: Vec<usize>,
    pub total_score: f64,
    pub total_cost: usize,
}

pub fn dominates<T: ObjectiveVector>(a: &T, b: &T) -> Result<bool, RoptError> {
    validate_pair(a, b)?;
    let mut strictly_better = false;
    for dimension in 0..a.objective_count() {
        let av = a.objective_value(dimension);
        let bv = b.objective_value(dimension);
        if av > bv {
            return Ok(false);
        }
        if av < bv {
            strictly_better = true;
        }
    }
    Ok(strictly_better)
}

pub fn fast_non_dominated_sort<T: ObjectiveVector>(
    objectives: &[T],
) -> Result<Vec<Vec<usize>>, RoptError> {
    validate_objectives(objectives)?;
    let n = objectives.len();
    if n == 0 {
        return Ok(vec![]);
    }

    let mut dominates_set: Vec<Vec<usize>> = vec![vec![]; n];
    let mut domination_count: Vec<usize> = vec![0; n];

    for p in 0..n {
        for q in 0..n {
            if p == q {
                continue;
            }
            if dominates(&objectives[p], &objectives[q])? {
                dominates_set[p].push(q);
            } else if dominates(&objectives[q], &objectives[p])? {
                domination_count[p] += 1;
            }
        }
    }

    let mut fronts = Vec::new();
    let mut current_front: Vec<usize> = (0..n).filter(|&p| domination_count[p] == 0).collect();

    while !current_front.is_empty() {
        fronts.push(current_front.clone());
        let mut next_front = Vec::new();
        for &p in &current_front {
            for &q in &dominates_set[p] {
                domination_count[q] -= 1;
                if domination_count[q] == 0 {
                    next_front.push(q);
                }
            }
        }
        current_front = next_front;
    }

    Ok(fronts)
}

pub fn crowding_distance<T: ObjectiveVector>(
    front: &[usize],
    objectives: &[T],
) -> Result<Vec<f64>, RoptError> {
    validate_objectives(objectives)?;
    validate_front(front, objectives.len())?;
    let n = front.len();
    if n == 0 {
        return Ok(vec![]);
    }
    if n <= 2 {
        return Ok(vec![f64::INFINITY; n]);
    }

    let dimensions = objectives[0].objective_count();
    let mut distances = vec![0.0; n];

    for dimension in 0..dimensions {
        let mut sorted: Vec<usize> = (0..n).collect();
        sorted.sort_by(|&a, &b| {
            let va = objectives[front[a]].objective_value(dimension);
            let vb = objectives[front[b]].objective_value(dimension);
            va.total_cmp(&vb)
        });

        distances[sorted[0]] = f64::INFINITY;
        distances[sorted[n - 1]] = f64::INFINITY;

        let obj_min = objectives[front[sorted[0]]].objective_value(dimension);
        let obj_max = objectives[front[sorted[n - 1]]].objective_value(dimension);
        let obj_range = obj_max - obj_min;
        validate_result("crowding objective range", obj_range)?;
        if obj_range == 0.0 {
            continue;
        }

        for i in 1..n - 1 {
            if distances[sorted[i]].is_infinite() {
                continue;
            }
            let prev = objectives[front[sorted[i - 1]]].objective_value(dimension);
            let next = objectives[front[sorted[i + 1]]].objective_value(dimension);
            let numerator = next - prev;
            validate_result("crowding objective span", numerator)?;
            let increment = numerator / obj_range;
            validate_result("crowding increment", increment)?;
            distances[sorted[i]] += increment;
            validate_result("crowding distance", distances[sorted[i]])?;
        }
    }

    Ok(distances)
}

pub fn derive_seed(domain: &[u8], parts: &[SeedPart]) -> Result<u64, RoptError> {
    if domain.is_empty() {
        return Err(RoptError::EmptySeedDomain);
    }
    let mut hasher = Sha256::new();
    hasher.update(b"ropt-seed-v2");
    hasher.update((domain.len() as u64).to_le_bytes());
    hasher.update(domain);
    hasher.update((parts.len() as u64).to_le_bytes());
    for part in parts {
        match part {
            SeedPart::U32(value) => {
                hasher.update([b'u', b'3', b'2']);
                hasher.update(value.to_le_bytes());
            }
            SeedPart::U64(value) => {
                hasher.update([b'u', b'6', b'4']);
                hasher.update(value.to_le_bytes());
            }
            SeedPart::Usize(value) => {
                hasher.update([b'u', b's', b'z']);
                hasher.update(value.to_le_bytes());
            }
        }
    }
    let digest = hasher.finalize();
    Ok(u64::from_le_bytes(
        digest[..8].try_into().expect("SHA-256 digest has 32 bytes"),
    ))
}

pub fn exact_budgeted_selection(
    items: &[BudgetItem],
    budget: usize,
) -> Result<BudgetSelection, RoptError> {
    validate_budget_items(items, budget)?;
    let mut states: Vec<Option<BudgetSelection>> = vec![None; budget + 1];
    states[0] = Some(BudgetSelection {
        selected_indices: Vec::new(),
        total_score: 0.0,
        total_cost: 0,
    });

    for (item_index, item) in items.iter().enumerate() {
        for capacity in (item.cost..=budget).rev() {
            let Some(previous) = states[capacity - item.cost].clone() else {
                continue;
            };
            let mut candidate = previous;
            candidate.selected_indices.push(item_index);
            candidate.total_score += item.score;
            validate_result("budgeted exact score", candidate.total_score)?;
            candidate.total_cost += item.cost;
            if is_better_selection(&candidate, states[capacity].as_ref()) {
                states[capacity] = Some(candidate);
            }
        }
    }

    states
        .into_iter()
        .flatten()
        .filter(|selection| !selection.selected_indices.is_empty())
        .max_by(compare_selections)
        .ok_or_else(|| impossible_budget(items, budget))
}

pub fn greedy_budgeted_selection(
    items: &[BudgetItem],
    budget: usize,
) -> Result<BudgetSelection, RoptError> {
    validate_budget_items(items, budget)?;
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by(|&left, &right| {
        let left_density = items[left].score / items[left].cost as f64;
        let right_density = items[right].score / items[right].cost as f64;
        right_density
            .total_cmp(&left_density)
            .then_with(|| items[right].score.total_cmp(&items[left].score))
            .then_with(|| items[left].cost.cmp(&items[right].cost))
            .then_with(|| left.cmp(&right))
    });

    let mut selection = BudgetSelection {
        selected_indices: Vec::new(),
        total_score: 0.0,
        total_cost: 0,
    };
    for item_index in order {
        let item = items[item_index];
        if selection.total_cost + item.cost > budget {
            continue;
        }
        selection.selected_indices.push(item_index);
        selection.total_cost += item.cost;
        selection.total_score += item.score;
        validate_result("budgeted greedy score", selection.total_score)?;
    }
    selection.selected_indices.sort_unstable();

    if selection.selected_indices.is_empty() {
        Err(impossible_budget(items, budget))
    } else {
        Ok(selection)
    }
}

fn validate_pair<T: ObjectiveVector>(a: &T, b: &T) -> Result<(), RoptError> {
    validate_objective(a, 0, a.objective_count())?;
    validate_objective(b, 1, a.objective_count())
}

fn validate_objectives<T: ObjectiveVector>(objectives: &[T]) -> Result<(), RoptError> {
    let Some(first) = objectives.first() else {
        return Ok(());
    };
    let expected = first.objective_count();
    for (point_index, objective) in objectives.iter().enumerate() {
        validate_objective(objective, point_index, expected)?;
    }
    Ok(())
}

fn validate_objective<T: ObjectiveVector>(
    objective: &T,
    point_index: usize,
    expected: usize,
) -> Result<(), RoptError> {
    let found = objective.objective_count();
    if found == 0 {
        return Err(RoptError::EmptyObjectiveVector { point_index });
    }
    if found != expected {
        return Err(RoptError::ObjectiveDimensionMismatch {
            point_index,
            expected,
            found,
        });
    }
    for dimension in 0..found {
        let value = objective.objective_value(dimension);
        if !value.is_finite() {
            return Err(RoptError::NonFiniteObjective {
                point_index,
                dimension,
                value,
            });
        }
    }
    Ok(())
}

fn validate_front(front: &[usize], len: usize) -> Result<(), RoptError> {
    let mut seen = vec![false; len];
    for (front_index, &point_index) in front.iter().enumerate() {
        if point_index >= len {
            return Err(RoptError::FrontIndexOutOfBounds {
                front_index,
                point_index,
                len,
            });
        }
        if seen[point_index] {
            return Err(RoptError::DuplicateFrontIndex {
                front_index,
                point_index,
            });
        }
        seen[point_index] = true;
    }
    Ok(())
}

fn validate_budget_items(items: &[BudgetItem], budget: usize) -> Result<(), RoptError> {
    if items.is_empty() {
        return Err(RoptError::EmptyBudgetItems);
    }
    if budget == 0 {
        return Err(RoptError::NonPositiveBudget);
    }
    for (item_index, item) in items.iter().enumerate() {
        if item.cost == 0 {
            return Err(RoptError::NonPositiveBudgetCost { item_index });
        }
        if !item.score.is_finite() || item.score < 0.0 {
            return Err(RoptError::InvalidBudgetScore {
                item_index,
                score: item.score,
            });
        }
    }
    Ok(())
}

fn is_better_selection(candidate: &BudgetSelection, incumbent: Option<&BudgetSelection>) -> bool {
    let Some(incumbent) = incumbent else {
        return true;
    };
    compare_selections(candidate, incumbent).is_gt()
}

fn compare_selections(left: &BudgetSelection, right: &BudgetSelection) -> std::cmp::Ordering {
    left.total_score
        .total_cmp(&right.total_score)
        .then_with(|| right.total_cost.cmp(&left.total_cost))
        .then_with(|| right.selected_indices.cmp(&left.selected_indices))
}

fn impossible_budget(items: &[BudgetItem], budget: usize) -> RoptError {
    let min_cost = items
        .iter()
        .map(|item| item.cost)
        .min()
        .expect("validated budget items are non-empty");
    RoptError::ImpossibleBudget { budget, min_cost }
}

fn validate_result(operation: &'static str, value: f64) -> Result<(), RoptError> {
    if !value.is_finite() {
        return Err(RoptError::NonFiniteResult { operation, value });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l0_dominance_requires_no_worse_and_one_strictly_better() {
        assert!(dominates(&[1.0, 1.0, 1.0], &[2.0, 1.0, 1.0]).unwrap());
        assert!(!dominates(&[1.0, 1.0, 1.0], &[1.0, 1.0, 1.0]).unwrap());
        assert!(!dominates(&[1.0, 2.0, 1.0], &[2.0, 1.0, 2.0]).unwrap());
    }

    #[test]
    fn l0_non_dominated_sort_orders_chain() {
        let objectives = [[1.0, 1.0], [2.0, 2.0], [3.0, 3.0]];

        assert_eq!(
            fast_non_dominated_sort(&objectives).unwrap(),
            vec![vec![0], vec![1], vec![2]]
        );
    }

    #[test]
    fn l0_crowding_marks_extremes_infinite() {
        let objectives = [[1.0, 1.0], [2.0, 2.0], [3.0, 3.0], [4.0, 4.0]];
        let distance = crowding_distance(&[0, 1, 2, 3], &objectives).unwrap();

        assert!(distance[0].is_infinite());
        assert!(distance[3].is_infinite());
        assert!(distance[1].is_finite());
        assert!(distance[2].is_finite());
    }

    #[test]
    fn l0_crowding_rejects_overflowed_objective_range() {
        let objectives = [[-f64::MAX], [0.0], [f64::MAX]];

        match crowding_distance(&[0, 1, 2], &objectives) {
            Err(RoptError::NonFiniteResult { operation, value }) => {
                assert_eq!(operation, "crowding objective range");
                assert!(value.is_infinite());
            }
            other => panic!("expected crowding overflow error, got {other:?}"),
        }
    }

    #[test]
    fn l0_dimension_mismatch_is_rejected() {
        assert_eq!(
            fast_non_dominated_sort(&[vec![1.0, 2.0], vec![1.0]]),
            Err(RoptError::ObjectiveDimensionMismatch {
                point_index: 1,
                expected: 2,
                found: 1
            })
        );
    }

    #[test]
    fn l0_non_finite_objective_is_rejected() {
        match crowding_distance(&[0], &[[f64::NAN]]) {
            Err(RoptError::NonFiniteObjective {
                point_index,
                dimension,
                value,
            }) => {
                assert_eq!((point_index, dimension), (0, 0));
                assert!(value.is_nan());
            }
            other => panic!("expected non-finite objective error, got {other:?}"),
        }
    }

    #[test]
    fn l0_duplicate_front_index_is_rejected() {
        assert_eq!(
            crowding_distance(&[0, 1, 1], &[[1.0], [2.0]]),
            Err(RoptError::DuplicateFrontIndex {
                front_index: 2,
                point_index: 1
            })
        );
    }

    #[test]
    fn l0_exact_budgeted_selection_finds_optimal_pack() {
        let items = [
            BudgetItem {
                score: 6.0,
                cost: 6,
            },
            BudgetItem {
                score: 4.0,
                cost: 4,
            },
            BudgetItem {
                score: 5.0,
                cost: 5,
            },
            BudgetItem {
                score: 7.0,
                cost: 7,
            },
        ];

        assert_eq!(
            exact_budgeted_selection(&items, 9).unwrap(),
            BudgetSelection {
                selected_indices: vec![1, 2],
                total_score: 9.0,
                total_cost: 9,
            }
        );
    }

    #[test]
    fn l0_exact_budgeted_selection_tie_breaks_by_lower_cost_then_index_order() {
        let items = [
            BudgetItem {
                score: 5.0,
                cost: 5,
            },
            BudgetItem {
                score: 5.0,
                cost: 4,
            },
            BudgetItem {
                score: 5.0,
                cost: 4,
            },
        ];

        assert_eq!(
            exact_budgeted_selection(&items, 5).unwrap(),
            BudgetSelection {
                selected_indices: vec![1],
                total_score: 5.0,
                total_cost: 4,
            }
        );
    }

    #[test]
    fn l0_greedy_budgeted_selection_is_deterministic_by_density() {
        let items = [
            BudgetItem {
                score: 6.0,
                cost: 6,
            },
            BudgetItem {
                score: 4.0,
                cost: 2,
            },
            BudgetItem {
                score: 3.0,
                cost: 2,
            },
        ];

        assert_eq!(
            greedy_budgeted_selection(&items, 4).unwrap(),
            BudgetSelection {
                selected_indices: vec![1, 2],
                total_score: 7.0,
                total_cost: 4,
            }
        );
    }

    #[test]
    fn l0_budgeted_selection_rejects_malformed_inputs() {
        assert_eq!(
            exact_budgeted_selection(&[], 10),
            Err(RoptError::EmptyBudgetItems)
        );
        assert_eq!(
            exact_budgeted_selection(
                &[BudgetItem {
                    score: 1.0,
                    cost: 1
                }],
                0
            ),
            Err(RoptError::NonPositiveBudget)
        );
        assert!(matches!(
            exact_budgeted_selection(
                &[BudgetItem {
                    score: f64::NAN,
                    cost: 1
                }],
                1
            ),
            Err(RoptError::InvalidBudgetScore {
                item_index: 0,
                score
            }) if score.is_nan()
        ));
        assert_eq!(
            exact_budgeted_selection(
                &[BudgetItem {
                    score: 1.0,
                    cost: 0
                }],
                1
            ),
            Err(RoptError::NonPositiveBudgetCost { item_index: 0 })
        );
        assert_eq!(
            exact_budgeted_selection(
                &[BudgetItem {
                    score: 1.0,
                    cost: 5
                }],
                4
            ),
            Err(RoptError::ImpossibleBudget {
                budget: 4,
                min_cost: 5
            })
        );
    }

    #[test]
    fn l0_seed_derivation_is_deterministic_and_domain_separated() {
        let a = derive_seed(b"PARETO_INIT_", &[SeedPart::U32(0), SeedPart::U64(42)]).unwrap();
        let b = derive_seed(b"PARETO_INIT_", &[SeedPart::U32(0), SeedPart::U64(42)]).unwrap();
        let c = derive_seed(b"PARETO_MUT_", &[SeedPart::U32(0), SeedPart::U64(42)]).unwrap();

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn l0_seed_derivation_tags_part_boundaries_and_types() {
        let raw_collision_value = u64::from_le_bytes([1, 0, 0, 0, b'_', 0, 0, 0]);

        let single_u64 =
            derive_seed(b"SEED_DOMAIN_", &[SeedPart::U64(raw_collision_value)]).unwrap();
        let split_u32s =
            derive_seed(b"SEED_DOMAIN_", &[SeedPart::U32(1), SeedPart::U32(0)]).unwrap();

        assert_ne!(single_u64, split_u32s);
    }

    #[test]
    fn l0_seed_derivation_tags_domain_boundary() {
        let with_part = derive_seed(b"SEED_DOMAIN_", &[SeedPart::U32(1)]).unwrap();
        let domain_absorbing_part = derive_seed(b"SEED_DOMAIN_u32\x01\x00\x00\x00", &[]).unwrap();

        assert_ne!(with_part, domain_absorbing_part);
    }

    #[test]
    fn l0_seed_derivation_rejects_empty_domain() {
        assert_eq!(
            derive_seed(b"", &[SeedPart::U64(42)]),
            Err(RoptError::EmptySeedDomain)
        );
    }

    #[test]
    fn l0_seed_derivation_supports_usize_parts() {
        let a = derive_seed(b"ENSEMBLE_CHAIN_", &[SeedPart::Usize(7), SeedPart::U64(42)]).unwrap();
        let b = derive_seed(b"ENSEMBLE_CHAIN_", &[SeedPart::Usize(7), SeedPart::U64(42)]).unwrap();

        assert_eq!(a, b);
    }
}
