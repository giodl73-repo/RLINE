use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const RCTX_VERSION: &str = "0.1";
pub const RCTX_CROSSWALK_HASH_PREFIX: &[u8] = b"RCTX_CROSSWALK_V1\0";
pub const RCTX_CROSSWALK_SET_HASH_PREFIX: &[u8] = b"RCTX_CROSSWALK_SET_V1\0";
pub const RCTX_PACKAGE_HASH_PREFIX: &[u8] = b"RCTX_PACKAGE_V1\0";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RctxCoreError {
    #[error("unsupported RCTX version: {0}")]
    UnsupportedVersion(String),
    #[error("package id is empty")]
    EmptyPackageId,
    #[error("package {package_id} has invalid package hash: {package_hash}")]
    InvalidPackageHash {
        package_id: String,
        package_hash: String,
    },
    #[error("context {context_hash} has invalid sha256 hash")]
    InvalidContextHash { context_hash: String },
    #[error("context {context_hash} has no units")]
    EmptyContextUnits { context_hash: String },
    #[error("context {context_hash} has duplicate unit id: {unit_id}")]
    DuplicateContextUnit {
        context_hash: String,
        unit_id: String,
    },
    #[error("duplicate source id: {source_id}")]
    DuplicateSourceId { source_id: String },
    #[error("source {source_id} has invalid sha256 hash: {sha256}")]
    InvalidSourceHash { source_id: String, sha256: String },
    #[error("crosswalk id is empty")]
    EmptyCrosswalkId,
    #[error("crosswalk {crosswalk_id} references missing context: {context_hash}")]
    MissingCrosswalkContext {
        crosswalk_id: String,
        context_hash: String,
    },
    #[error(
        "crosswalk {crosswalk_id} references missing unit {unit_id} in context {context_hash}"
    )]
    MissingCrosswalkUnit {
        crosswalk_id: String,
        context_hash: String,
        unit_id: String,
    },
    #[error("crosswalk {crosswalk_id} has invalid rational weight")]
    InvalidCrosswalkWeight { crosswalk_id: String },
    #[error(
        "duplicate crosswalk row for {crosswalk_id} {from_unit_id}->{to_unit_id} {weight_kind:?}"
    )]
    DuplicateCrosswalkRow {
        crosswalk_id: String,
        from_unit_id: String,
        to_unit_id: String,
        weight_kind: CrosswalkWeightKind,
    },
    #[error("crosswalk {crosswalk_id} exhaustive weights for {from_unit_id} do not sum to 1")]
    CrosswalkWeightSum {
        crosswalk_id: String,
        from_unit_id: String,
    },
    #[error("crosswalk {crosswalk_id} references missing source: {source_id}")]
    MissingSourceRef {
        crosswalk_id: String,
        source_id: String,
    },
    #[error("graph {graph_id} references missing context: {context_hash}")]
    GraphMissingContext {
        graph_id: String,
        context_hash: String,
    },
    #[error("graph {graph_id} has invalid graph hash: {graph_hash}")]
    InvalidGraphHash {
        graph_id: String,
        graph_hash: String,
    },
    #[error("graph {graph_id} references missing source: {source_id}")]
    GraphMissingSourceRef { graph_id: String, source_id: String },
    #[error("claim boundary package id does not match manifest")]
    ClaimBoundaryPackageMismatch,
    #[error("claim boundary must include proves and does_not_prove entries")]
    EmptyClaimBoundary,
    #[error("canonical JSON error: {0}")]
    CanonicalJson(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RctxPackage {
    pub manifest: RctxManifest,
    #[serde(default)]
    pub source_index: Vec<RctxSourceIndexEntry>,
    #[serde(default)]
    pub units: Vec<ContextUnitIndex>,
    #[serde(default)]
    pub graphs: Vec<GraphRecord>,
    #[serde(default)]
    pub crosswalks: Vec<CrosswalkRecord>,
    pub claim_boundary: ClaimBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RctxManifest {
    pub rctx_version: String,
    pub package_id: String,
    pub jurisdiction: String,
    pub producer: String,
    pub created_at: String,
    pub package_content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RctxSourceIndexEntry {
    pub source_id: String,
    pub path: String,
    pub sha256: String,
    pub media_type: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextUnitIndex {
    pub context_hash: String,
    pub unit_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceIndexEntry {
    pub source_id: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphRecord {
    pub graph_id: String,
    pub context_hash: String,
    pub graph_hash: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
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
    pub from_context_hash: String,
    pub to_context_hash: String,
    pub from_unit_id: String,
    pub to_unit_id: String,
    pub weight: RationalWeight,
    pub weight_kind: CrosswalkWeightKind,
    pub exhaustive: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosswalkVerificationInput {
    #[serde(default)]
    pub contexts: Vec<ContextUnitIndex>,
    #[serde(default)]
    pub sources: Vec<SourceIndexEntry>,
    #[serde(default)]
    pub crosswalks: Vec<CrosswalkRecord>,
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

pub fn package_content_hash(package: &RctxPackage) -> Result<String, RctxCoreError> {
    let mut manifest = serde_json::to_value(&package.manifest)
        .map_err(|err| RctxCoreError::CanonicalJson(err.to_string()))?;
    if let Some(object) = manifest.as_object_mut() {
        object.remove("package_content_hash");
    }
    let value = serde_json::json!({
        "rctx_version": package.manifest.rctx_version,
        "manifest_without_package_content_hash": manifest,
        "source_index": package.source_index,
        "units": package.units,
        "graphs": package.graphs,
        "crosswalks": package.crosswalks,
        "claim_boundary": package.claim_boundary,
    });
    canonical_hash(RCTX_PACKAGE_HASH_PREFIX, &value)
}

pub fn verify_package(package: &RctxPackage) -> Result<Vec<VerificationReport>, RctxCoreError> {
    verify_manifest(package)?;
    let sources = verify_package_sources(package)?;
    let input = CrosswalkVerificationInput {
        contexts: package.units.clone(),
        sources: sources.iter().cloned().collect(),
        crosswalks: package.crosswalks.clone(),
    };
    let mut reports = vec![report("manifest_package_hash")];
    reports.extend(verify_crosswalk_input(&input)?);
    verify_graphs(package, &sources)?;
    if !package.graphs.is_empty() {
        reports.push(report("graph_context_refs"));
        reports.push(report("graph_source_refs"));
    }
    verify_claim_boundary(package)?;
    reports.push(report("claim_boundary_present"));
    Ok(reports)
}

pub fn verify_crosswalk_input(
    input: &CrosswalkVerificationInput,
) -> Result<Vec<VerificationReport>, RctxCoreError> {
    let contexts = context_unit_sets(&input.contexts)?;
    let sources = source_ids(&input.sources)?;
    verify_crosswalk_records(&input.crosswalks, &contexts, &sources)?;
    let mut reports = vec![report("context_unit_index")];
    if !input.sources.is_empty() {
        reports.push(report("source_index"));
    }
    if !input.crosswalks.is_empty() {
        reports.push(report("crosswalk_unit_refs"));
        reports.push(report("crosswalk_weight_sum"));
        reports.push(report("crosswalk_source_refs"));
    }
    Ok(reports)
}

pub fn crosswalk_record_hash(record: &CrosswalkRecord) -> Result<String, RctxCoreError> {
    let value = serde_json::to_value(record)
        .map_err(|err| RctxCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(RCTX_CROSSWALK_HASH_PREFIX, &value)
}

pub fn crosswalk_set_hash(records: &[CrosswalkRecord]) -> Result<String, RctxCoreError> {
    let value = serde_json::to_value(records)
        .map_err(|err| RctxCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(RCTX_CROSSWALK_SET_HASH_PREFIX, &value)
}

pub fn synthetic_minimal_package_fixture() -> Result<RctxPackage, RctxCoreError> {
    let context_hash = "sha256:b11f1eabcaf33e2d2691ddbe498c650830cffb9b0fb62820292d4ca0166c0bb7";
    let source_hash = "sha256:253e748527f192efece9361b79250aaa1e1e00348f89d44a3e7dc3267433b3cf";
    let graph_hash = "sha256:3333333333333333333333333333333333333333333333333333333333333333";
    let mut package = RctxPackage {
        manifest: RctxManifest {
            rctx_version: RCTX_VERSION.to_string(),
            package_id: "syn-rctx-l0-shared-context".to_string(),
            jurisdiction: "SYN".to_string(),
            producer: "rctx-fixture".to_string(),
            created_at: "2026-05-13T00:00:00Z".to_string(),
            package_content_hash: zero_hash(),
        },
        source_index: vec![RctxSourceIndexEntry {
            source_id: "syn-precincts.csv".to_string(),
            path: "sources/syn-precincts.csv".to_string(),
            sha256: source_hash.to_string(),
            media_type: "text/csv".to_string(),
            description: "Synthetic precinct unit list".to_string(),
        }],
        units: vec![ContextUnitIndex {
            context_hash: context_hash.to_string(),
            unit_ids: vec![
                "syn:precinct:P-001".to_string(),
                "syn:precinct:P-002".to_string(),
            ],
        }],
        graphs: vec![GraphRecord {
            graph_id: "graph:summary-basic-adjacency".to_string(),
            context_hash: context_hash.to_string(),
            graph_hash: graph_hash.to_string(),
            source_refs: vec!["syn-precincts.csv".to_string()],
        }],
        crosswalks: vec![
            CrosswalkRecord {
                crosswalk_id: "cw-summary-basic-identity".to_string(),
                from_context_hash: context_hash.to_string(),
                to_context_hash: context_hash.to_string(),
                from_unit_id: "syn:precinct:P-001".to_string(),
                to_unit_id: "syn:precinct:P-001".to_string(),
                weight: RationalWeight { num: 1, den: 1 },
                weight_kind: CrosswalkWeightKind::UnitCount,
                exhaustive: true,
                source_refs: vec!["syn-precincts.csv".to_string()],
            },
            CrosswalkRecord {
                crosswalk_id: "cw-summary-basic-identity".to_string(),
                from_context_hash: context_hash.to_string(),
                to_context_hash: context_hash.to_string(),
                from_unit_id: "syn:precinct:P-002".to_string(),
                to_unit_id: "syn:precinct:P-002".to_string(),
                weight: RationalWeight { num: 1, den: 1 },
                weight_kind: CrosswalkWeightKind::UnitCount,
                exhaustive: true,
                source_refs: vec!["syn-precincts.csv".to_string()],
            },
        ],
        claim_boundary: ClaimBoundary {
            package_id: "syn-rctx-l0-shared-context".to_string(),
            proves: vec![
                "declared canonical unit ids are unique".to_string(),
                "declared graph and crosswalk records reference known context and source ids"
                    .to_string(),
                "declared exhaustive crosswalk weights sum to one".to_string(),
            ],
            does_not_prove: vec![
                "official legal validity of geography".to_string(),
                "completeness of all geography sources".to_string(),
                "vote totals, district assignments, or rendered maps".to_string(),
            ],
            caveats: Vec::new(),
        },
    };
    package.manifest.package_content_hash = package_content_hash(&package)?;
    Ok(package)
}

pub fn synthetic_missing_source_ref_package_fixture() -> Result<RctxPackage, RctxCoreError> {
    let mut package = synthetic_minimal_package_fixture()?;
    package.crosswalks[0].source_refs = vec!["source:missing".to_string()];
    package.manifest.package_content_hash = package_content_hash(&package)?;
    Ok(package)
}

pub fn synthetic_ai_context_package_fixture() -> Result<RctxPackage, RctxCoreError> {
    let context_hash = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    let repo_source_hash =
        "sha256:d111111111111111111111111111111111111111111111111111111111111111";
    let note_source_hash =
        "sha256:d222222222222222222222222222222222222222222222222222222222222222";
    let graph_hash = "sha256:d333333333333333333333333333333333333333333333333333333333333333";
    let mut package = RctxPackage {
        manifest: RctxManifest {
            rctx_version: RCTX_VERSION.to_string(),
            package_id: "crop-ai-context-fixture".to_string(),
            jurisdiction: "AI-CONTEXT".to_string(),
            producer: "crop-fixture".to_string(),
            created_at: "2026-05-15T00:00:00Z".to_string(),
            package_content_hash: zero_hash(),
        },
        source_index: vec![
            RctxSourceIndexEntry {
                source_id: "repo:README.md".to_string(),
                path: "repo/README.md".to_string(),
                sha256: repo_source_hash.to_string(),
                media_type: "text/markdown".to_string(),
                description: "Repository overview used as canonical context".to_string(),
            },
            RctxSourceIndexEntry {
                source_id: "note:architecture.md".to_string(),
                path: "notes/architecture.md".to_string(),
                sha256: note_source_hash.to_string(),
                media_type: "text/markdown".to_string(),
                description: "Architecture note that bridges repo modules".to_string(),
            },
        ],
        units: vec![ContextUnitIndex {
            context_hash: context_hash.to_string(),
            unit_ids: vec![
                "unit:repo-readme:overview".to_string(),
                "unit:note-architecture:bridge".to_string(),
                "unit:issue-17:decision".to_string(),
            ],
        }],
        graphs: vec![GraphRecord {
            graph_id: "graph:crop-evidence-neighborhood".to_string(),
            context_hash: context_hash.to_string(),
            graph_hash: graph_hash.to_string(),
            source_refs: vec![
                "repo:README.md".to_string(),
                "note:architecture.md".to_string(),
            ],
        }],
        crosswalks: Vec::new(),
        claim_boundary: ClaimBoundary {
            package_id: "crop-ai-context-fixture".to_string(),
            proves: vec![
                "declared AI context units have stable identifiers".to_string(),
                "declared evidence graph references known context and source ids".to_string(),
                "package hash is deterministic for replayable context packs".to_string(),
            ],
            does_not_prove: vec![
                "LLM answer correctness".to_string(),
                "semantic completeness of every repository file".to_string(),
                "embedding quality or model-specific retrieval behavior".to_string(),
            ],
            caveats: vec![
                "fixture is intentionally domain-generic and contains no election geography"
                    .to_string(),
            ],
        },
    };
    package.manifest.package_content_hash = package_content_hash(&package)?;
    Ok(package)
}

pub fn synthetic_bad_crosswalk_weight_package_fixture() -> Result<RctxPackage, RctxCoreError> {
    let mut package = synthetic_minimal_package_fixture()?;
    package.crosswalks[0].weight = RationalWeight { num: 2, den: 1 };
    package.manifest.package_content_hash = package_content_hash(&package)?;
    Ok(package)
}

pub fn canonical_hash(prefix: &[u8], value: &Value) -> Result<String, RctxCoreError> {
    let canonical = canonicalize_value(value);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| RctxCoreError::CanonicalJson(err.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(prefix);
    hasher.update(bytes);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

pub fn is_sha256_hash(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit()))
}

fn context_unit_sets(
    contexts: &[ContextUnitIndex],
) -> Result<BTreeMap<String, BTreeSet<String>>, RctxCoreError> {
    let mut map = BTreeMap::new();
    for context in contexts {
        if !is_sha256_hash(&context.context_hash) {
            return Err(RctxCoreError::InvalidContextHash {
                context_hash: context.context_hash.clone(),
            });
        }
        if context.unit_ids.is_empty() {
            return Err(RctxCoreError::EmptyContextUnits {
                context_hash: context.context_hash.clone(),
            });
        }
        let mut units = BTreeSet::new();
        for unit_id in &context.unit_ids {
            if !units.insert(unit_id.clone()) {
                return Err(RctxCoreError::DuplicateContextUnit {
                    context_hash: context.context_hash.clone(),
                    unit_id: unit_id.clone(),
                });
            }
        }
        map.insert(context.context_hash.clone(), units);
    }
    Ok(map)
}

fn source_ids(sources: &[SourceIndexEntry]) -> Result<BTreeSet<String>, RctxCoreError> {
    let mut ids = BTreeSet::new();
    for source in sources {
        if !ids.insert(source.source_id.clone()) {
            return Err(RctxCoreError::DuplicateSourceId {
                source_id: source.source_id.clone(),
            });
        }
        if !is_sha256_hash(&source.sha256) {
            return Err(RctxCoreError::InvalidSourceHash {
                source_id: source.source_id.clone(),
                sha256: source.sha256.clone(),
            });
        }
    }
    Ok(ids)
}

fn verify_manifest(package: &RctxPackage) -> Result<(), RctxCoreError> {
    if package.manifest.rctx_version != RCTX_VERSION {
        return Err(RctxCoreError::UnsupportedVersion(
            package.manifest.rctx_version.clone(),
        ));
    }
    if package.manifest.package_id.trim().is_empty() {
        return Err(RctxCoreError::EmptyPackageId);
    }
    if !is_sha256_hash(&package.manifest.package_content_hash) {
        return Err(RctxCoreError::InvalidPackageHash {
            package_id: package.manifest.package_id.clone(),
            package_hash: package.manifest.package_content_hash.clone(),
        });
    }
    Ok(())
}

fn verify_package_sources(
    package: &RctxPackage,
) -> Result<BTreeSet<SourceIndexEntry>, RctxCoreError> {
    let sources = package
        .source_index
        .iter()
        .map(|source| SourceIndexEntry {
            source_id: source.source_id.clone(),
            sha256: source.sha256.clone(),
        })
        .collect::<Vec<_>>();
    source_ids(&sources)?;
    Ok(sources.into_iter().collect())
}

fn verify_graphs(
    package: &RctxPackage,
    sources: &BTreeSet<SourceIndexEntry>,
) -> Result<(), RctxCoreError> {
    let context_hashes: BTreeSet<&str> = package
        .units
        .iter()
        .map(|context| context.context_hash.as_str())
        .collect();
    let source_ids: BTreeSet<&str> = sources
        .iter()
        .map(|source| source.source_id.as_str())
        .collect();
    for graph in &package.graphs {
        if !context_hashes.contains(graph.context_hash.as_str()) {
            return Err(RctxCoreError::GraphMissingContext {
                graph_id: graph.graph_id.clone(),
                context_hash: graph.context_hash.clone(),
            });
        }
        if !is_sha256_hash(&graph.graph_hash) {
            return Err(RctxCoreError::InvalidGraphHash {
                graph_id: graph.graph_id.clone(),
                graph_hash: graph.graph_hash.clone(),
            });
        }
        for source_id in &graph.source_refs {
            if !source_ids.contains(source_id.as_str()) {
                return Err(RctxCoreError::GraphMissingSourceRef {
                    graph_id: graph.graph_id.clone(),
                    source_id: source_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn verify_claim_boundary(package: &RctxPackage) -> Result<(), RctxCoreError> {
    if package.claim_boundary.package_id != package.manifest.package_id {
        return Err(RctxCoreError::ClaimBoundaryPackageMismatch);
    }
    if package.claim_boundary.proves.is_empty() || package.claim_boundary.does_not_prove.is_empty()
    {
        return Err(RctxCoreError::EmptyClaimBoundary);
    }
    Ok(())
}

fn verify_crosswalk_records(
    crosswalks: &[CrosswalkRecord],
    contexts: &BTreeMap<String, BTreeSet<String>>,
    sources: &BTreeSet<String>,
) -> Result<(), RctxCoreError> {
    let mut seen = BTreeSet::new();
    let mut sums: BTreeMap<(String, String, CrosswalkWeightKind), RationalSum> = BTreeMap::new();
    for crosswalk in crosswalks {
        if crosswalk.crosswalk_id.trim().is_empty() {
            return Err(RctxCoreError::EmptyCrosswalkId);
        }
        ensure_context_unit(
            &crosswalk.crosswalk_id,
            &crosswalk.from_context_hash,
            &crosswalk.from_unit_id,
            contexts,
        )?;
        ensure_context_unit(
            &crosswalk.crosswalk_id,
            &crosswalk.to_context_hash,
            &crosswalk.to_unit_id,
            contexts,
        )?;
        if crosswalk.weight.den <= 0 || crosswalk.weight.num < 0 {
            return Err(RctxCoreError::InvalidCrosswalkWeight {
                crosswalk_id: crosswalk.crosswalk_id.clone(),
            });
        }
        if !seen.insert((
            crosswalk.crosswalk_id.as_str(),
            crosswalk.from_unit_id.as_str(),
            crosswalk.to_unit_id.as_str(),
            crosswalk.weight_kind,
        )) {
            return Err(RctxCoreError::DuplicateCrosswalkRow {
                crosswalk_id: crosswalk.crosswalk_id.clone(),
                from_unit_id: crosswalk.from_unit_id.clone(),
                to_unit_id: crosswalk.to_unit_id.clone(),
                weight_kind: crosswalk.weight_kind,
            });
        }
        for source_id in &crosswalk.source_refs {
            if !sources.contains(source_id) {
                return Err(RctxCoreError::MissingSourceRef {
                    crosswalk_id: crosswalk.crosswalk_id.clone(),
                    source_id: source_id.clone(),
                });
            }
        }
        if crosswalk.exhaustive {
            sums.entry((
                crosswalk.crosswalk_id.clone(),
                crosswalk.from_unit_id.clone(),
                crosswalk.weight_kind,
            ))
            .or_default()
            .add(crosswalk.weight);
        }
    }

    for ((crosswalk_id, from_unit_id, _), sum) in sums {
        if !sum.is_one() {
            return Err(RctxCoreError::CrosswalkWeightSum {
                crosswalk_id,
                from_unit_id,
            });
        }
    }
    Ok(())
}

fn ensure_context_unit(
    crosswalk_id: &str,
    context_hash: &str,
    unit_id: &str,
    contexts: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), RctxCoreError> {
    let units =
        contexts
            .get(context_hash)
            .ok_or_else(|| RctxCoreError::MissingCrosswalkContext {
                crosswalk_id: crosswalk_id.to_string(),
                context_hash: context_hash.to_string(),
            })?;
    if !units.contains(unit_id) {
        return Err(RctxCoreError::MissingCrosswalkUnit {
            crosswalk_id: crosswalk_id.to_string(),
            context_hash: context_hash.to_string(),
            unit_id: unit_id.to_string(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
struct RationalSum {
    num: i128,
    den: i128,
}

impl RationalSum {
    fn add(&mut self, value: RationalWeight) {
        if self.den == 0 {
            self.num = value.num as i128;
            self.den = value.den as i128;
            return;
        }
        self.num = self.num * value.den as i128 + value.num as i128 * self.den;
        self.den *= value.den as i128;
        let divisor = gcd(self.num.unsigned_abs(), self.den.unsigned_abs()) as i128;
        if divisor > 1 {
            self.num /= divisor;
            self.den /= divisor;
        }
    }

    fn is_one(self) -> bool {
        self.den != 0 && self.num == self.den
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

fn report(check_id: &'static str) -> VerificationReport {
    VerificationReport { check_id }
}

fn zero_hash() -> String {
    "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string()
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_value).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), canonicalize_value(&map[key]));
            }
            Value::Object(sorted)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    const FROM_HASH: &str =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    const TO_HASH: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
    const SOURCE_HASH: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn valid_input() -> CrosswalkVerificationInput {
        CrosswalkVerificationInput {
            contexts: vec![
                ContextUnitIndex {
                    context_hash: FROM_HASH.to_string(),
                    unit_ids: vec!["P-001".to_string(), "P-002".to_string()],
                },
                ContextUnitIndex {
                    context_hash: TO_HASH.to_string(),
                    unit_ids: vec!["D-01".to_string(), "D-02".to_string()],
                },
            ],
            sources: vec![SourceIndexEntry {
                source_id: "source:crosswalk".to_string(),
                sha256: SOURCE_HASH.to_string(),
            }],
            crosswalks: vec![
                CrosswalkRecord {
                    crosswalk_id: "cw-precinct-district".to_string(),
                    from_context_hash: FROM_HASH.to_string(),
                    to_context_hash: TO_HASH.to_string(),
                    from_unit_id: "P-001".to_string(),
                    to_unit_id: "D-01".to_string(),
                    weight: RationalWeight { num: 1, den: 3 },
                    weight_kind: CrosswalkWeightKind::Population,
                    exhaustive: true,
                    source_refs: vec!["source:crosswalk".to_string()],
                },
                CrosswalkRecord {
                    crosswalk_id: "cw-precinct-district".to_string(),
                    from_context_hash: FROM_HASH.to_string(),
                    to_context_hash: TO_HASH.to_string(),
                    from_unit_id: "P-001".to_string(),
                    to_unit_id: "D-02".to_string(),
                    weight: RationalWeight { num: 2, den: 3 },
                    weight_kind: CrosswalkWeightKind::Population,
                    exhaustive: true,
                    source_refs: vec!["source:crosswalk".to_string()],
                },
            ],
        }
    }

    #[test]
    fn verifies_exhaustive_crosswalk_weight_sum() {
        let reports = verify_crosswalk_input(&valid_input()).expect("valid crosswalk verifies");
        assert!(reports
            .iter()
            .any(|report| report.check_id == "crosswalk_weight_sum"));
    }

    #[test]
    fn rejects_crosswalk_weight_sum_drift() {
        let mut input = valid_input();
        input.crosswalks[1].weight = RationalWeight { num: 1, den: 3 };

        assert!(matches!(
            verify_crosswalk_input(&input),
            Err(RctxCoreError::CrosswalkWeightSum { .. })
        ));
    }

    #[test]
    fn rejects_missing_endpoint_unit() {
        let mut input = valid_input();
        input.crosswalks[0].to_unit_id = "D-99".to_string();

        assert!(matches!(
            verify_crosswalk_input(&input),
            Err(RctxCoreError::MissingCrosswalkUnit { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_crosswalk_row() {
        let mut input = valid_input();
        input.crosswalks[1].to_unit_id = "D-01".to_string();

        assert!(matches!(
            verify_crosswalk_input(&input),
            Err(RctxCoreError::DuplicateCrosswalkRow { .. })
        ));
    }

    #[test]
    fn rejects_missing_source_ref() {
        let mut input = valid_input();
        input.crosswalks[0].source_refs = vec!["source:missing".to_string()];

        assert!(matches!(
            verify_crosswalk_input(&input),
            Err(RctxCoreError::MissingSourceRef { .. })
        ));
    }

    #[test]
    fn crosswalk_hash_is_domain_prefixed_and_stable() {
        let input = valid_input();
        let first = crosswalk_record_hash(&input.crosswalks[0]).unwrap();
        let second = crosswalk_record_hash(&input.crosswalks[0]).unwrap();
        assert_eq!(first, second);
        assert!(is_sha256_hash(&first));
    }

    #[test]
    fn crosswalk_set_hash_is_stable() {
        let input = valid_input();
        let first = crosswalk_set_hash(&input.crosswalks).unwrap();
        let second = crosswalk_set_hash(&input.crosswalks).unwrap();
        assert_eq!(first, second);
        assert!(is_sha256_hash(&first));
    }

    #[test]
    fn minimal_package_fixture_verifies() {
        let package = synthetic_minimal_package_fixture().unwrap();
        let reports = verify_package(&package).expect("minimal RCTX package verifies");
        assert!(reports
            .iter()
            .any(|report| report.check_id == "manifest_package_hash"));
        assert!(reports
            .iter()
            .any(|report| report.check_id == "crosswalk_source_refs"));
        assert!(reports
            .iter()
            .any(|report| report.check_id == "graph_source_refs"));
        assert!(reports
            .iter()
            .any(|report| report.check_id == "claim_boundary_present"));
    }

    #[test]
    fn ai_context_package_fixture_verifies_without_election_assumptions() {
        let package = synthetic_ai_context_package_fixture().unwrap();
        let reports = verify_package(&package).expect("AI context RCTX package verifies");

        assert_eq!(package.manifest.jurisdiction, "AI-CONTEXT");
        assert!(package.crosswalks.is_empty());
        assert_eq!(
            package.units[0].unit_ids[1],
            "unit:note-architecture:bridge"
        );
        assert!(package
            .claim_boundary
            .does_not_prove
            .iter()
            .any(|claim| { claim == "embedding quality or model-specific retrieval behavior" }));
        assert!(reports
            .iter()
            .any(|report| report.check_id == "graph_source_refs"));
        assert!(reports
            .iter()
            .any(|report| report.check_id == "claim_boundary_present"));
    }

    #[test]
    fn minimal_package_rejects_missing_source_ref() {
        let package = synthetic_missing_source_ref_package_fixture().unwrap();
        assert!(matches!(
            verify_package(&package),
            Err(RctxCoreError::MissingSourceRef { .. })
        ));
    }

    #[test]
    fn minimal_package_rejects_bad_crosswalk_weight() {
        let package = synthetic_bad_crosswalk_weight_package_fixture().unwrap();
        assert!(matches!(
            verify_package(&package),
            Err(RctxCoreError::CrosswalkWeightSum { .. })
        ));
    }

    #[test]
    fn package_content_hash_has_rctx_prefix_and_ignores_declared_hash() {
        let mut package = synthetic_minimal_package_fixture().unwrap();
        let hash = package_content_hash(&package).unwrap();
        assert!(is_sha256_hash(&hash));

        package.manifest.package_content_hash =
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
        assert_eq!(package_content_hash(&package).unwrap(), hash);
    }

    #[test]
    fn docs_fixture_matches_helper_and_source_hashes() {
        let helper = synthetic_minimal_package_fixture().unwrap();
        let fixture = read_fixture("l0-shared-context");
        assert_eq!(fixture, helper);
        assert_eq!(
            package_content_hash(&fixture).unwrap(),
            fixture.manifest.package_content_hash
        );
        assert_eq!(
            crosswalk_set_hash(&fixture.crosswalks).unwrap(),
            read_json::<serde_json::Value>(
                &fixture_root("l0-shared-context")
                    .join("proofs")
                    .join("package-hashes.json"),
            )["crosswalk_set_hash"]
                .as_str()
                .unwrap()
        );
        verify_package(&fixture).expect("docs fixture verifies");

        let root = fixture_root("l0-shared-context");
        for source in fixture.source_index {
            let bytes = std::fs::read(root.join(&source.path)).expect("read source bytes");
            assert_eq!(source.sha256, sha256_string(&bytes));
        }
    }

    fn read_fixture(name: &str) -> RctxPackage {
        let root = fixture_root(name);
        RctxPackage {
            manifest: read_json(&root.join("manifest.json")),
            source_index: read_json(&root.join("sources").join("source-index.json")),
            units: read_ndjson(&root.join("units").join("context-units.ndjson")),
            graphs: read_ndjson(&root.join("graphs").join("graphs.ndjson")),
            crosswalks: read_ndjson(&root.join("units").join("crosswalks.ndjson")),
            claim_boundary: read_json(&root.join("claims").join("claim-boundary.json")),
        }
    }

    fn fixture_root(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("fixtures")
            .join("rctx")
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

    fn sha256_string(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        format!("sha256:{:x}", h.finalize())
    }
}
