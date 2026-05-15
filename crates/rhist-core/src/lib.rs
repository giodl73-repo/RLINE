use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const RHIST_VERSION: &str = "0.1";
pub const RHIST_PACKAGE_HASH_PREFIX: &[u8] = b"RHIST_PACKAGE_V1\0";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RhistCoreError {
    #[error("unsupported RHIST version: {0}")]
    UnsupportedVersion(String),
    #[error("package id is empty")]
    EmptyPackageId,
    #[error("package {package_id} has invalid package hash: {package_hash}")]
    InvalidPackageHash {
        package_id: String,
        package_hash: String,
    },
    #[error("package {package_id} hash mismatch: declared {declared}, computed {computed}")]
    PackageHashMismatch {
        package_id: String,
        declared: String,
        computed: String,
    },
    #[error("manifest references missing cycle: {cycle_id}")]
    ManifestMissingCycle { cycle_id: String },
    #[error("duplicate source id: {source_id}")]
    DuplicateSourceId { source_id: String },
    #[error("source {source_id} has invalid sha256: {sha256}")]
    InvalidSourceHash { source_id: String, sha256: String },
    #[error("source ref is missing: {source_id}")]
    MissingSourceRef { source_id: String },
    #[error("duplicate context id: {context_id}")]
    DuplicateContextId { context_id: String },
    #[error("context {context_id} references missing cycle: {cycle_id}")]
    ContextMissingCycle {
        context_id: String,
        cycle_id: String,
    },
    #[error("context {context_id} has invalid context hash: {context_hash}")]
    InvalidContextHash {
        context_id: String,
        context_hash: String,
    },
    #[error("context {context_id} has no unit ids")]
    EmptyContextUnits { context_id: String },
    #[error("context {context_id} has duplicate unit id: {unit_id}")]
    DuplicateContextUnit { context_id: String, unit_id: String },
    #[error("duplicate cycle id: {cycle_id}")]
    DuplicateCycleId { cycle_id: String },
    #[error("cycle {cycle_id} references missing context: {context_id}")]
    CycleMissingContext {
        cycle_id: String,
        context_id: String,
    },
    #[error("cycle {cycle_id} context hash mismatch")]
    CycleContextHashMismatch { cycle_id: String },
    #[error("duplicate lineage event id: {event_id}")]
    DuplicateLineageEventId { event_id: String },
    #[error("lineage event {event_id} references missing cycle: {cycle_id}")]
    LineageMissingCycle { event_id: String, cycle_id: String },
    #[error("lineage event {event_id} references missing unit {unit_id} in cycle {cycle_id}")]
    LineageMissingUnit {
        event_id: String,
        cycle_id: String,
        unit_id: String,
    },
    #[error("lineage event {event_id} has invalid cardinality for {event_kind:?}")]
    InvalidLineageCardinality {
        event_id: String,
        event_kind: LineageEventKind,
    },
    #[error(
        "duplicate crosswalk row for {crosswalk_id} {from_unit_id}->{to_unit_id} {weight_kind:?}"
    )]
    DuplicateCrosswalkRow {
        crosswalk_id: String,
        from_unit_id: String,
        to_unit_id: String,
        weight_kind: CrosswalkWeightKind,
    },
    #[error("crosswalk id is empty")]
    EmptyCrosswalkId,
    #[error("crosswalk {crosswalk_id} references missing cycle: {cycle_id}")]
    CrosswalkMissingCycle {
        crosswalk_id: String,
        cycle_id: String,
    },
    #[error("crosswalk {crosswalk_id} references missing unit {unit_id} in cycle {cycle_id}")]
    CrosswalkMissingUnit {
        crosswalk_id: String,
        cycle_id: String,
        unit_id: String,
    },
    #[error("crosswalk {crosswalk_id} references context hash not declared for cycle {cycle_id}")]
    CrosswalkContextHashMismatch {
        crosswalk_id: String,
        cycle_id: String,
    },
    #[error("crosswalk {crosswalk_id} has invalid rational weight")]
    InvalidCrosswalkWeight { crosswalk_id: String },
    #[error("crosswalk {crosswalk_id} exhaustive weights for {from_unit_id} do not sum to 1")]
    CrosswalkWeightSum {
        crosswalk_id: String,
        from_unit_id: String,
    },
    #[error("claim boundary package id does not match manifest")]
    ClaimBoundaryPackageMismatch,
    #[error("claim boundary must include proves and does_not_prove entries")]
    EmptyClaimBoundary,
    #[error("canonical JSON error: {0}")]
    CanonicalJson(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RhistPackage {
    pub manifest: RhistManifest,
    #[serde(default)]
    pub source_index: Vec<SourceIndexEntry>,
    #[serde(default)]
    pub context_index: Vec<ContextIndexEntry>,
    #[serde(default)]
    pub cycles: Vec<CycleRecord>,
    #[serde(default)]
    pub lineage_events: Vec<LineageEvent>,
    #[serde(default)]
    pub crosswalks: Vec<CrosswalkRecord>,
    pub claim_boundary: ClaimBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RhistManifest {
    pub rhist_version: String,
    pub package_id: String,
    pub jurisdiction: String,
    pub cycle_ids: Vec<String>,
    pub producer: String,
    pub created_at: String,
    pub package_content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIndexEntry {
    pub source_id: String,
    pub path: String,
    pub sha256: String,
    pub media_type: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextIndexEntry {
    pub context_id: String,
    pub context_hash: String,
    pub rctx_version: String,
    pub unit_kind: String,
    pub cycle_id: String,
    pub unit_ids: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CycleKind {
    GeneralElection,
    PrimaryElection,
    SpecialElection,
    RedistrictingCycle,
    AdministrativeSnapshot,
    Imported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleRecord {
    pub cycle_id: String,
    pub jurisdiction: String,
    pub cycle_kind: CycleKind,
    pub effective_date: String,
    pub context_id: String,
    pub context_hash: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LineageEventKind {
    Unchanged,
    Create,
    Close,
    Rename,
    Split,
    Merge,
    BoundaryChange,
    AdministrativeRecode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LineageConfidence {
    Official,
    Derived,
    ManualReview,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageEvent {
    pub event_id: String,
    pub event_kind: LineageEventKind,
    pub from_cycle_id: String,
    pub to_cycle_id: String,
    #[serde(default)]
    pub from_unit_ids: Vec<String>,
    #[serde(default)]
    pub to_unit_ids: Vec<String>,
    pub effective_date: String,
    pub authority: String,
    pub confidence: LineageConfidence,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrosswalkWeightKind {
    Population,
    Area,
    Ballots,
    RegisteredVoters,
    Manual,
    UnitCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalWeight {
    pub num: i64,
    pub den: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosswalkRecord {
    pub crosswalk_id: String,
    pub from_cycle_id: String,
    pub to_cycle_id: String,
    pub from_context_hash: String,
    pub to_context_hash: String,
    pub from_unit_id: String,
    pub to_unit_id: String,
    pub weight: RationalWeight,
    pub weight_kind: CrosswalkWeightKind,
    pub exhaustive: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimBoundary {
    pub package_id: String,
    pub proves: Vec<String>,
    pub does_not_prove: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    pub check_id: &'static str,
}

pub fn package_content_hash(package: &RhistPackage) -> Result<String, RhistCoreError> {
    let mut manifest = serde_json::to_value(&package.manifest)
        .map_err(|err| RhistCoreError::CanonicalJson(err.to_string()))?;
    if let Some(object) = manifest.as_object_mut() {
        object.remove("package_content_hash");
    }
    let value = serde_json::json!({
        "rhist_version": package.manifest.rhist_version,
        "manifest_without_package_content_hash": manifest,
        "source_index": package.source_index,
        "context_index": package.context_index,
        "cycles": package.cycles,
        "lineage_events": package.lineage_events,
        "crosswalks": package.crosswalks,
        "claim_boundary": package.claim_boundary,
    });
    canonical_hash(RHIST_PACKAGE_HASH_PREFIX, &value)
}

pub fn canonical_hash(prefix: &[u8], value: &Value) -> Result<String, RhistCoreError> {
    let canonical = canonicalize_value(value);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| RhistCoreError::CanonicalJson(err.to_string()))?;
    let mut h = Sha256::new();
    h.update(prefix);
    h.update(bytes);
    Ok(format!("sha256:{:x}", h.finalize()))
}

pub fn verify_package(package: &RhistPackage) -> Result<Vec<VerificationReport>, RhistCoreError> {
    let mut reports = Vec::new();

    verify_manifest(package)?;
    reports.push(report("manifest_cycle_refs"));

    let source_ids = verify_source_index(package)?;
    verify_all_source_refs(package, &source_ids)?;
    reports.push(report("source_refs_resolve"));

    let cycle_ids = cycle_id_set(&package.cycles)?;
    let contexts_by_cycle = verify_context_index(package, &cycle_ids)?;
    reports.push(report("context_cycle_refs"));
    reports.push(report("context_unit_ids_unique"));

    verify_cycles(package)?;
    reports.push(report("cycle_context_refs"));

    verify_lineage_events(package, &cycle_ids, &contexts_by_cycle)?;
    reports.push(report("lineage_unit_refs"));
    reports.push(report("lineage_cardinality"));

    verify_crosswalks(package, &cycle_ids, &contexts_by_cycle)?;
    if !package.crosswalks.is_empty() {
        reports.push(report("crosswalk_unit_refs"));
        reports.push(report("crosswalk_weight_sum"));
    }

    verify_claim_boundary(package)?;
    reports.push(report("claim_boundary_present"));

    Ok(reports)
}

fn verify_manifest(package: &RhistPackage) -> Result<(), RhistCoreError> {
    if package.manifest.rhist_version != RHIST_VERSION {
        return Err(RhistCoreError::UnsupportedVersion(
            package.manifest.rhist_version.clone(),
        ));
    }
    if package.manifest.package_id.trim().is_empty() {
        return Err(RhistCoreError::EmptyPackageId);
    }
    if !is_sha256_hash(&package.manifest.package_content_hash) {
        return Err(RhistCoreError::InvalidPackageHash {
            package_id: package.manifest.package_id.clone(),
            package_hash: package.manifest.package_content_hash.clone(),
        });
    }
    let computed = package_content_hash(package)?;
    if computed != package.manifest.package_content_hash {
        return Err(RhistCoreError::PackageHashMismatch {
            package_id: package.manifest.package_id.clone(),
            declared: package.manifest.package_content_hash.clone(),
            computed,
        });
    }
    let cycles: BTreeSet<&str> = package
        .cycles
        .iter()
        .map(|cycle| cycle.cycle_id.as_str())
        .collect();
    for cycle_id in &package.manifest.cycle_ids {
        if !cycles.contains(cycle_id.as_str()) {
            return Err(RhistCoreError::ManifestMissingCycle {
                cycle_id: cycle_id.clone(),
            });
        }
    }
    Ok(())
}

fn verify_source_index(package: &RhistPackage) -> Result<BTreeSet<String>, RhistCoreError> {
    let mut ids = BTreeSet::new();
    for source in &package.source_index {
        if !ids.insert(source.source_id.clone()) {
            return Err(RhistCoreError::DuplicateSourceId {
                source_id: source.source_id.clone(),
            });
        }
        if !is_sha256_hash(&source.sha256) {
            return Err(RhistCoreError::InvalidSourceHash {
                source_id: source.source_id.clone(),
                sha256: source.sha256.clone(),
            });
        }
    }
    Ok(ids)
}

fn verify_all_source_refs(
    package: &RhistPackage,
    source_ids: &BTreeSet<String>,
) -> Result<(), RhistCoreError> {
    for source_ref in package
        .context_index
        .iter()
        .flat_map(|context| context.source_refs.iter())
        .chain(
            package
                .cycles
                .iter()
                .flat_map(|cycle| cycle.source_refs.iter()),
        )
        .chain(
            package
                .lineage_events
                .iter()
                .flat_map(|event| event.source_refs.iter()),
        )
        .chain(
            package
                .crosswalks
                .iter()
                .flat_map(|crosswalk| crosswalk.source_refs.iter()),
        )
    {
        if !source_ids.contains(source_ref) {
            return Err(RhistCoreError::MissingSourceRef {
                source_id: source_ref.clone(),
            });
        }
    }
    Ok(())
}

fn cycle_id_set(cycles: &[CycleRecord]) -> Result<BTreeSet<String>, RhistCoreError> {
    let mut ids = BTreeSet::new();
    for cycle in cycles {
        if !ids.insert(cycle.cycle_id.clone()) {
            return Err(RhistCoreError::DuplicateCycleId {
                cycle_id: cycle.cycle_id.clone(),
            });
        }
    }
    Ok(ids)
}

fn verify_context_index(
    package: &RhistPackage,
    cycle_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, BTreeSet<String>>, RhistCoreError> {
    let mut context_ids = BTreeSet::new();
    let mut contexts_by_cycle = BTreeMap::new();
    for context in &package.context_index {
        if !context_ids.insert(context.context_id.as_str()) {
            return Err(RhistCoreError::DuplicateContextId {
                context_id: context.context_id.clone(),
            });
        }
        if !cycle_ids.contains(&context.cycle_id) {
            return Err(RhistCoreError::ContextMissingCycle {
                context_id: context.context_id.clone(),
                cycle_id: context.cycle_id.clone(),
            });
        }
        if !is_sha256_hash(&context.context_hash) {
            return Err(RhistCoreError::InvalidContextHash {
                context_id: context.context_id.clone(),
                context_hash: context.context_hash.clone(),
            });
        }
        if context.unit_ids.is_empty() {
            return Err(RhistCoreError::EmptyContextUnits {
                context_id: context.context_id.clone(),
            });
        }
        let mut units = BTreeSet::new();
        for unit_id in &context.unit_ids {
            if !units.insert(unit_id.clone()) {
                return Err(RhistCoreError::DuplicateContextUnit {
                    context_id: context.context_id.clone(),
                    unit_id: unit_id.clone(),
                });
            }
        }
        contexts_by_cycle.insert(context.cycle_id.clone(), units);
    }
    Ok(contexts_by_cycle)
}

fn verify_cycles(package: &RhistPackage) -> Result<(), RhistCoreError> {
    let contexts: BTreeMap<&str, &ContextIndexEntry> = package
        .context_index
        .iter()
        .map(|context| (context.context_id.as_str(), context))
        .collect();
    for cycle in &package.cycles {
        let Some(context) = contexts.get(cycle.context_id.as_str()) else {
            return Err(RhistCoreError::CycleMissingContext {
                cycle_id: cycle.cycle_id.clone(),
                context_id: cycle.context_id.clone(),
            });
        };
        if cycle.context_hash != context.context_hash {
            return Err(RhistCoreError::CycleContextHashMismatch {
                cycle_id: cycle.cycle_id.clone(),
            });
        }
    }
    Ok(())
}

fn verify_lineage_events(
    package: &RhistPackage,
    cycle_ids: &BTreeSet<String>,
    contexts_by_cycle: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), RhistCoreError> {
    let mut event_ids = BTreeSet::new();
    for event in &package.lineage_events {
        if !event_ids.insert(event.event_id.as_str()) {
            return Err(RhistCoreError::DuplicateLineageEventId {
                event_id: event.event_id.clone(),
            });
        }
        ensure_cycle(&event.event_id, &event.from_cycle_id, cycle_ids)?;
        ensure_cycle(&event.event_id, &event.to_cycle_id, cycle_ids)?;
        ensure_units(
            &event.event_id,
            &event.from_cycle_id,
            &event.from_unit_ids,
            contexts_by_cycle,
        )?;
        ensure_units(
            &event.event_id,
            &event.to_cycle_id,
            &event.to_unit_ids,
            contexts_by_cycle,
        )?;
        if !valid_lineage_cardinality(
            event.event_kind,
            event.from_unit_ids.len(),
            event.to_unit_ids.len(),
        ) {
            return Err(RhistCoreError::InvalidLineageCardinality {
                event_id: event.event_id.clone(),
                event_kind: event.event_kind,
            });
        }
    }
    Ok(())
}

fn verify_crosswalks(
    package: &RhistPackage,
    cycle_ids: &BTreeSet<String>,
    contexts_by_cycle: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), RhistCoreError> {
    for crosswalk in &package.crosswalks {
        if !cycle_ids.contains(&crosswalk.from_cycle_id) {
            return Err(RhistCoreError::CrosswalkMissingCycle {
                crosswalk_id: crosswalk.crosswalk_id.clone(),
                cycle_id: crosswalk.from_cycle_id.clone(),
            });
        }
        if !cycle_ids.contains(&crosswalk.to_cycle_id) {
            return Err(RhistCoreError::CrosswalkMissingCycle {
                crosswalk_id: crosswalk.crosswalk_id.clone(),
                cycle_id: crosswalk.to_cycle_id.clone(),
            });
        }
        let from_cycle = package
            .cycles
            .iter()
            .find(|cycle| cycle.cycle_id == crosswalk.from_cycle_id)
            .expect("cycle existence checked above");
        if from_cycle.context_hash != crosswalk.from_context_hash {
            return Err(RhistCoreError::CrosswalkContextHashMismatch {
                crosswalk_id: crosswalk.crosswalk_id.clone(),
                cycle_id: crosswalk.from_cycle_id.clone(),
            });
        }
        let to_cycle = package
            .cycles
            .iter()
            .find(|cycle| cycle.cycle_id == crosswalk.to_cycle_id)
            .expect("cycle existence checked above");
        if to_cycle.context_hash != crosswalk.to_context_hash {
            return Err(RhistCoreError::CrosswalkContextHashMismatch {
                crosswalk_id: crosswalk.crosswalk_id.clone(),
                cycle_id: crosswalk.to_cycle_id.clone(),
            });
        }
    }

    let input = rctx_core::CrosswalkVerificationInput {
        contexts: package
            .context_index
            .iter()
            .map(|context| rctx_core::ContextUnitIndex {
                context_hash: context.context_hash.clone(),
                unit_ids: context.unit_ids.clone(),
            })
            .collect(),
        sources: package
            .source_index
            .iter()
            .map(|source| rctx_core::SourceIndexEntry {
                source_id: source.source_id.clone(),
                sha256: source.sha256.clone(),
            })
            .collect(),
        crosswalks: package.crosswalks.iter().map(to_rctx_crosswalk).collect(),
    };
    rctx_core::verify_crosswalk_input(&input)
        .map(|_| ())
        .map_err(|err| map_rctx_crosswalk_error(err, package, contexts_by_cycle))?;
    Ok(())
}

fn verify_claim_boundary(package: &RhistPackage) -> Result<(), RhistCoreError> {
    if package.claim_boundary.package_id != package.manifest.package_id {
        return Err(RhistCoreError::ClaimBoundaryPackageMismatch);
    }
    if package.claim_boundary.proves.is_empty() || package.claim_boundary.does_not_prove.is_empty()
    {
        return Err(RhistCoreError::EmptyClaimBoundary);
    }
    Ok(())
}

fn ensure_cycle(
    event_id: &str,
    cycle_id: &str,
    cycle_ids: &BTreeSet<String>,
) -> Result<(), RhistCoreError> {
    if !cycle_ids.contains(cycle_id) {
        return Err(RhistCoreError::LineageMissingCycle {
            event_id: event_id.to_string(),
            cycle_id: cycle_id.to_string(),
        });
    }
    Ok(())
}

fn ensure_units(
    event_id: &str,
    cycle_id: &str,
    unit_ids: &[String],
    contexts_by_cycle: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), RhistCoreError> {
    let Some(units) = contexts_by_cycle.get(cycle_id) else {
        return Ok(());
    };
    for unit_id in unit_ids {
        if !units.contains(unit_id) {
            return Err(RhistCoreError::LineageMissingUnit {
                event_id: event_id.to_string(),
                cycle_id: cycle_id.to_string(),
                unit_id: unit_id.clone(),
            });
        }
    }
    Ok(())
}

fn valid_lineage_cardinality(kind: LineageEventKind, from_len: usize, to_len: usize) -> bool {
    match kind {
        LineageEventKind::Unchanged
        | LineageEventKind::Rename
        | LineageEventKind::AdministrativeRecode => from_len == 1 && to_len == 1,
        LineageEventKind::Split => from_len == 1 && to_len >= 2,
        LineageEventKind::Merge => from_len >= 2 && to_len == 1,
        LineageEventKind::BoundaryChange => from_len >= 1 && to_len >= 1,
        LineageEventKind::Create => from_len == 0 && to_len >= 1,
        LineageEventKind::Close => from_len >= 1 && to_len == 0,
    }
}

fn is_sha256_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_value).collect()),
        Value::Object(map) => canonicalize_object(map),
        other => other.clone(),
    }
}

fn canonicalize_object(map: &Map<String, Value>) -> Value {
    let mut sorted = Map::new();
    let mut keys: Vec<_> = map.keys().collect();
    keys.sort();
    for key in keys {
        sorted.insert(key.clone(), canonicalize_value(&map[key]));
    }
    Value::Object(sorted)
}

fn report(check_id: &'static str) -> VerificationReport {
    VerificationReport { check_id }
}

fn to_rctx_crosswalk(crosswalk: &CrosswalkRecord) -> rctx_core::CrosswalkRecord {
    rctx_core::CrosswalkRecord {
        crosswalk_id: crosswalk.crosswalk_id.clone(),
        from_context_hash: crosswalk.from_context_hash.clone(),
        to_context_hash: crosswalk.to_context_hash.clone(),
        from_unit_id: crosswalk.from_unit_id.clone(),
        to_unit_id: crosswalk.to_unit_id.clone(),
        weight: rctx_core::RationalWeight {
            num: crosswalk.weight.num,
            den: crosswalk.weight.den,
        },
        weight_kind: to_rctx_weight_kind(crosswalk.weight_kind),
        exhaustive: crosswalk.exhaustive,
        source_refs: crosswalk.source_refs.clone(),
    }
}

fn to_rctx_weight_kind(kind: CrosswalkWeightKind) -> rctx_core::CrosswalkWeightKind {
    match kind {
        CrosswalkWeightKind::Population => rctx_core::CrosswalkWeightKind::Population,
        CrosswalkWeightKind::Area => rctx_core::CrosswalkWeightKind::Area,
        CrosswalkWeightKind::Ballots => rctx_core::CrosswalkWeightKind::Ballots,
        CrosswalkWeightKind::RegisteredVoters => rctx_core::CrosswalkWeightKind::RegisteredVoters,
        CrosswalkWeightKind::Manual => rctx_core::CrosswalkWeightKind::Manual,
        CrosswalkWeightKind::UnitCount => rctx_core::CrosswalkWeightKind::UnitCount,
    }
}

fn from_rctx_weight_kind(kind: rctx_core::CrosswalkWeightKind) -> CrosswalkWeightKind {
    match kind {
        rctx_core::CrosswalkWeightKind::Population => CrosswalkWeightKind::Population,
        rctx_core::CrosswalkWeightKind::Area => CrosswalkWeightKind::Area,
        rctx_core::CrosswalkWeightKind::Ballots => CrosswalkWeightKind::Ballots,
        rctx_core::CrosswalkWeightKind::RegisteredVoters => CrosswalkWeightKind::RegisteredVoters,
        rctx_core::CrosswalkWeightKind::Manual => CrosswalkWeightKind::Manual,
        rctx_core::CrosswalkWeightKind::UnitCount => CrosswalkWeightKind::UnitCount,
    }
}

fn map_rctx_crosswalk_error(
    err: rctx_core::RctxCoreError,
    package: &RhistPackage,
    contexts_by_cycle: &BTreeMap<String, BTreeSet<String>>,
) -> RhistCoreError {
    match err {
        rctx_core::RctxCoreError::EmptyCrosswalkId => RhistCoreError::EmptyCrosswalkId,
        rctx_core::RctxCoreError::DuplicateCrosswalkRow {
            crosswalk_id,
            from_unit_id,
            to_unit_id,
            weight_kind,
        } => RhistCoreError::DuplicateCrosswalkRow {
            crosswalk_id,
            from_unit_id,
            to_unit_id,
            weight_kind: from_rctx_weight_kind(weight_kind),
        },
        rctx_core::RctxCoreError::InvalidCrosswalkWeight { crosswalk_id } => {
            RhistCoreError::InvalidCrosswalkWeight { crosswalk_id }
        }
        rctx_core::RctxCoreError::CrosswalkWeightSum {
            crosswalk_id,
            from_unit_id,
        } => RhistCoreError::CrosswalkWeightSum {
            crosswalk_id,
            from_unit_id,
        },
        rctx_core::RctxCoreError::MissingSourceRef { source_id, .. } => {
            RhistCoreError::MissingSourceRef { source_id }
        }
        rctx_core::RctxCoreError::MissingCrosswalkUnit {
            crosswalk_id,
            context_hash,
            unit_id,
        } => {
            let cycle_id =
                cycle_for_context_hash(package, &context_hash, &unit_id, contexts_by_cycle)
                    .unwrap_or(context_hash);
            RhistCoreError::CrosswalkMissingUnit {
                crosswalk_id,
                cycle_id,
                unit_id,
            }
        }
        rctx_core::RctxCoreError::MissingCrosswalkContext {
            crosswalk_id,
            context_hash,
        } => RhistCoreError::CrosswalkContextHashMismatch {
            crosswalk_id,
            cycle_id: context_hash,
        },
        rctx_core::RctxCoreError::InvalidContextHash { context_hash } => {
            RhistCoreError::CrosswalkContextHashMismatch {
                crosswalk_id: String::new(),
                cycle_id: context_hash,
            }
        }
        rctx_core::RctxCoreError::EmptyContextUnits { context_hash }
        | rctx_core::RctxCoreError::DuplicateContextUnit { context_hash, .. } => {
            RhistCoreError::CrosswalkContextHashMismatch {
                crosswalk_id: String::new(),
                cycle_id: context_hash,
            }
        }
        rctx_core::RctxCoreError::DuplicateSourceId { source_id }
        | rctx_core::RctxCoreError::InvalidSourceHash { source_id, .. } => {
            RhistCoreError::MissingSourceRef { source_id }
        }
        rctx_core::RctxCoreError::UnsupportedVersion(version) => RhistCoreError::CanonicalJson(
            format!("unexpected RCTX package version error: {version}"),
        ),
        rctx_core::RctxCoreError::EmptyPackageId => {
            RhistCoreError::CanonicalJson("unexpected empty RCTX package id".to_string())
        }
        rctx_core::RctxCoreError::InvalidPackageHash {
            package_id,
            package_hash,
        } => RhistCoreError::CanonicalJson(format!(
            "unexpected RCTX package hash error for {package_id}: {package_hash}"
        )),
        rctx_core::RctxCoreError::GraphMissingContext {
            graph_id,
            context_hash,
        } => RhistCoreError::CanonicalJson(format!(
            "unexpected RCTX graph context error for {graph_id}: {context_hash}"
        )),
        rctx_core::RctxCoreError::InvalidGraphHash {
            graph_id,
            graph_hash,
        } => RhistCoreError::CanonicalJson(format!(
            "unexpected RCTX graph hash error for {graph_id}: {graph_hash}"
        )),
        rctx_core::RctxCoreError::GraphMissingSourceRef {
            graph_id,
            source_id,
        } => RhistCoreError::CanonicalJson(format!(
            "unexpected RCTX graph source error for {graph_id}: {source_id}"
        )),
        rctx_core::RctxCoreError::ClaimBoundaryPackageMismatch => RhistCoreError::CanonicalJson(
            "unexpected RCTX claim-boundary package mismatch".to_string(),
        ),
        rctx_core::RctxCoreError::EmptyClaimBoundary => {
            RhistCoreError::CanonicalJson("unexpected empty RCTX claim boundary".to_string())
        }
        rctx_core::RctxCoreError::CanonicalJson(message) => RhistCoreError::CanonicalJson(message),
    }
}

fn cycle_for_context_hash(
    package: &RhistPackage,
    context_hash: &str,
    unit_id: &str,
    contexts_by_cycle: &BTreeMap<String, BTreeSet<String>>,
) -> Option<String> {
    package
        .cycles
        .iter()
        .filter(|cycle| cycle.context_hash == context_hash)
        .find(|cycle| {
            contexts_by_cycle
                .get(&cycle.cycle_id)
                .is_some_and(|units| !units.contains(unit_id))
        })
        .map(|cycle| cycle.cycle_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn l0_rename_fixture_verifies() {
        let package = read_fixture("l0-rename");
        let reports = verify_package(&package).expect("l0 rename fixture should verify");
        assert!(reports
            .iter()
            .any(|report| report.check_id == "lineage_unit_refs"));
        assert!(reports
            .iter()
            .any(|report| report.check_id == "lineage_cardinality"));
    }

    #[test]
    fn l0_missing_unit_fixture_fails_lineage_unit_refs() {
        let package = read_fixture("l0-missing-unit");
        let err = verify_package(&package).expect_err("missing unit fixture should fail");
        assert!(matches!(err, RhistCoreError::LineageMissingUnit { .. }));
    }

    #[test]
    fn l1_split_merge_fixture_verifies_crosswalk_weights() {
        let package = read_fixture("l1-split-merge");
        let reports = verify_package(&package).expect("l1 split/merge fixture should verify");
        assert!(reports
            .iter()
            .any(|report| report.check_id == "crosswalk_unit_refs"));
        assert!(reports
            .iter()
            .any(|report| report.check_id == "crosswalk_weight_sum"));
    }

    #[test]
    fn l2_three_cycle_fixture_verifies() {
        let package = read_fixture("l2-three-cycle");
        let reports = verify_package(&package).expect("l2 three-cycle fixture should verify");
        assert_eq!(package.cycles.len(), 3);
        assert_eq!(package.lineage_events.len(), 7);
        assert_eq!(package.crosswalks.len(), 9);
        assert!(reports
            .iter()
            .any(|report| report.check_id == "crosswalk_weight_sum"));
    }

    #[test]
    fn l2_three_cycle_fixture_locks_rename_split_and_merge() {
        let package = read_fixture("l2-three-cycle");
        let kinds: BTreeSet<_> = package
            .lineage_events
            .iter()
            .map(|event| event.event_kind)
            .collect();
        assert!(kinds.contains(&LineageEventKind::Rename));
        assert!(kinds.contains(&LineageEventKind::Split));
        assert!(kinds.contains(&LineageEventKind::Merge));

        verify_package(&package).expect("rename/split/merge fixture must verify");
    }

    #[test]
    fn real_ri_tract_unchanged_fixture_verifies() {
        let package = read_fixture("real-ri-tract-unchanged");
        let reports =
            verify_package(&package).expect("real RI tract pressure fixture should verify");
        assert_eq!(package.cycles.len(), 3);
        assert_eq!(package.lineage_events.len(), 2);
        assert_eq!(package.crosswalks.len(), 2);
        assert!(reports
            .iter()
            .any(|report| report.check_id == "source_refs_resolve"));
    }

    #[test]
    fn l1_bad_weights_fixture_fails_crosswalk_weight_sum() {
        let package = read_fixture("l1-bad-weights");
        let err = verify_package(&package).expect_err("bad weights fixture should fail");
        assert!(matches!(err, RhistCoreError::CrosswalkWeightSum { .. }));
    }

    #[test]
    fn package_content_hash_has_rhist_prefix_and_ignores_declared_hash() {
        let mut package = read_fixture("l1-split-merge");
        let hash = package_content_hash(&package).unwrap();
        assert!(hash.starts_with("sha256:"));
        assert_ne!(
            hash,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        );

        package.manifest.package_content_hash =
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
        assert_eq!(package_content_hash(&package).unwrap(), hash);
    }

    #[test]
    fn package_content_hash_changes_when_lineage_changes() {
        let mut package = read_fixture("l0-rename");
        let before = package_content_hash(&package).unwrap();
        package.lineage_events[0].explanation.push_str(" Changed.");
        let after = package_content_hash(&package).unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn verifier_rejects_manifest_package_hash_drift() {
        let mut package = read_fixture("l2-three-cycle");
        package.manifest.package_content_hash =
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();

        assert!(matches!(
            verify_package(&package),
            Err(RhistCoreError::PackageHashMismatch { .. })
        ));
    }

    #[test]
    fn fixture_source_hashes_match_preserved_bytes() {
        for fixture in [
            "l0-rename",
            "l0-missing-unit",
            "l1-split-merge",
            "l1-bad-weights",
            "l2-three-cycle",
            "real-ri-tract-unchanged",
        ] {
            let root = fixture_root(fixture);
            let source_index: Vec<SourceIndexEntry> =
                read_json(&root.join("sources").join("source-index.json"));
            for source in source_index {
                let bytes = std::fs::read(root.join(&source.path)).expect("read source bytes");
                assert_eq!(source.sha256, sha256_string(&bytes));
            }
        }
    }

    #[test]
    fn invalid_rename_cardinality_is_rejected() {
        let mut package = read_fixture("l0-rename");
        package.lineage_events[0]
            .to_unit_ids
            .push("syn:precinct:P-001B".to_string());
        package.manifest.package_content_hash = package_content_hash(&package).unwrap();
        let err = verify_package(&package).expect_err("bad rename cardinality should fail");
        assert!(matches!(
            err,
            RhistCoreError::LineageMissingUnit { .. }
                | RhistCoreError::InvalidLineageCardinality { .. }
        ));
    }

    fn read_fixture(name: &str) -> RhistPackage {
        let root = fixture_root(name);
        RhistPackage {
            manifest: read_json(&root.join("manifest.json")),
            source_index: read_json(&root.join("sources").join("source-index.json")),
            context_index: read_ndjson(&root.join("contexts").join("context-index.ndjson")),
            cycles: read_ndjson(&root.join("units").join("cycles.ndjson")),
            lineage_events: read_ndjson(&root.join("units").join("lineage-events.ndjson")),
            crosswalks: read_optional_ndjson(&root.join("units").join("crosswalks.ndjson")),
            claim_boundary: read_json(&root.join("claims").join("claim-boundary.json")),
        }
    }

    fn fixture_root(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("fixtures")
            .join("rhist")
            .join(name)
    }

    fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
        let text = std::fs::read_to_string(path).expect("read json");
        serde_json::from_str(&text).expect("parse json")
    }

    fn read_ndjson<T: for<'de> Deserialize<'de>>(path: &Path) -> Vec<T> {
        let text = std::fs::read_to_string(path).expect("read ndjson");
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("parse ndjson row"))
            .collect()
    }

    fn read_optional_ndjson<T: for<'de> Deserialize<'de>>(path: &Path) -> Vec<T> {
        if !path.exists() {
            return Vec::new();
        }
        read_ndjson(path)
    }

    fn sha256_string(bytes: &[u8]) -> String {
        let digest = Sha256::digest(bytes);
        let mut hex = String::with_capacity(64);
        for byte in digest {
            hex.push_str(&format!("{byte:02x}"));
        }
        format!("sha256:{hex}")
    }
}
