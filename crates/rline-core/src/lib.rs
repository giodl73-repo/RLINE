use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const RLINE_MANIFEST_SCHEMA: &str = "rline.manifest.v1";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RlineError {
    #[error("unsupported RLINE manifest schema: {0}")]
    UnsupportedSchema(String),
    #[error("manifest family is empty")]
    EmptyFamily,
    #[error("crate name is empty")]
    EmptyCrateName,
    #[error("crate {crate_name} has empty source path")]
    EmptySourcePath { crate_name: String },
    #[error("duplicate crate name: {crate_name}")]
    DuplicateCrateName { crate_name: String },
    #[error("crate {crate_name} references unknown internal dependency: {dependency}")]
    UnknownInternalDependency {
        crate_name: String,
        dependency: String,
    },
    #[error("consumer repo is empty")]
    EmptyConsumerRepo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlineManifest {
    pub schema_version: String,
    pub family: String,
    pub crates: Vec<KernelCrateSpec>,
    #[serde(default)]
    pub consumers: Vec<ConsumerRef>,
    #[serde(default)]
    pub non_goals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelCrateSpec {
    pub name: String,
    pub kind: KernelKind,
    pub source_path: String,
    #[serde(default)]
    pub internal_dependencies: Vec<String>,
    pub migration_status: MigrationStatus,
    pub public_contract: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KernelKind {
    Context,
    Graph,
    Statistics,
    Math,
    Optimization,
    Facility,
    History,
    HistoryIo,
    HistoryCli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationStatus {
    Candidate,
    Planned,
    Extracted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumerRef {
    pub repo: String,
    pub role: String,
}

pub fn foundation_manifest() -> RlineManifest {
    RlineManifest {
        schema_version: RLINE_MANIFEST_SCHEMA.to_string(),
        family: "rline".to_string(),
        crates: vec![
            KernelCrateSpec {
                name: "rctx-core".to_string(),
                kind: KernelKind::Context,
                source_path: "crates\\rctx-core".to_string(),
                internal_dependencies: vec![],
                migration_status: MigrationStatus::Extracted,
                public_contract: "context packages, crosswalk verification, graph/source provenance records".to_string(),
            },
            KernelCrateSpec {
                name: "rstat-core".to_string(),
                kind: KernelKind::Statistics,
                source_path: "crates\\rstat-core".to_string(),
                internal_dependencies: vec![],
                migration_status: MigrationStatus::Extracted,
                public_contract: "deterministic summary statistics, quantiles, weighted samples, and interval helpers".to_string(),
            },
            KernelCrateSpec {
                name: "rmath-core".to_string(),
                kind: KernelKind::Math,
                source_path: "crates\\rmath-core".to_string(),
                internal_dependencies: vec![],
                migration_status: MigrationStatus::Extracted,
                public_contract: "deterministic numeric and linear algebra kernels".to_string(),
            },
            KernelCrateSpec {
                name: "ropt-core".to_string(),
                kind: KernelKind::Optimization,
                source_path: "crates\\ropt-core".to_string(),
                internal_dependencies: vec![],
                migration_status: MigrationStatus::Extracted,
                public_contract: "multi-objective sorting, crowding distance, deterministic seed derivation, and budget selection".to_string(),
            },
            KernelCrateSpec {
                name: "rgraph-core".to_string(),
                kind: KernelKind::Graph,
                source_path: "crates\\rgraph-core".to_string(),
                internal_dependencies: vec![
                    "ropt-core".to_string(),
                    "rstat-core".to_string(),
                ],
                migration_status: MigrationStatus::Extracted,
                public_contract: "directed weighted graph traits, shortest paths, cut metrics, connectivity, and cluster summaries".to_string(),
            },
            KernelCrateSpec {
                name: "rfacility-core".to_string(),
                kind: KernelKind::Facility,
                source_path: "crates\\rfacility-core".to_string(),
                internal_dependencies: vec![],
                migration_status: MigrationStatus::Extracted,
                public_contract: "product-neutral facility identity, category, capability, and requirement primitives".to_string(),
            },
            KernelCrateSpec {
                name: "rhist-core".to_string(),
                kind: KernelKind::History,
                source_path: "crates\\rhist-core".to_string(),
                internal_dependencies: vec!["rctx-core".to_string()],
                migration_status: MigrationStatus::Extracted,
                public_contract: "history and lineage primitives layered on RCTX context packages".to_string(),
            },
            KernelCrateSpec {
                name: "rhist-io".to_string(),
                kind: KernelKind::HistoryIo,
                source_path: "crates\\rhist-io".to_string(),
                internal_dependencies: vec!["rhist-core".to_string()],
                migration_status: MigrationStatus::Extracted,
                public_contract: "RHIST package directory read/write helpers".to_string(),
            },
            KernelCrateSpec {
                name: "rhist-cli".to_string(),
                kind: KernelKind::HistoryCli,
                source_path: "crates\\rhist-cli".to_string(),
                internal_dependencies: vec!["rhist-core".to_string(), "rhist-io".to_string()],
                migration_status: MigrationStatus::Extracted,
                public_contract: "standalone RHIST command-line verifier".to_string(),
            },
        ],
        consumers: vec![
            ConsumerRef {
                repo: "BISECT".to_string(),
                role: "current source repo and future downstream consumer".to_string(),
            },
            ConsumerRef {
                repo: "CROP".to_string(),
                role: "uses shared graph/context kernels without depending on BISECT".to_string(),
            },
            ConsumerRef {
                repo: "ROUTE".to_string(),
                role: "candidate consumer for graph and optimization kernels".to_string(),
            },
            ConsumerRef {
                repo: "FLETCH".to_string(),
                role: "parallel shared infrastructure repo; no dependency required".to_string(),
            },
            ConsumerRef {
                repo: "RPLAN".to_string(),
                role: "sibling district-plan package family".to_string(),
            },
            ConsumerRef {
                repo: "RCOUNT".to_string(),
                role: "sibling election-count package family consuming RCTX/RHIST kernels".to_string(),
            },
        ],
        non_goals: vec![
            "Do not move BISECT redistricting product logic into RLINE.".to_string(),
            "Do not move RCOUNT election-audit workflow logic into RLINE.".to_string(),
            "Do not move RPLAN district-plan package workflow logic into RLINE.".to_string(),
            "Do not make RLINE depend on CROP, FLETCH, ROUTE, or BISECT application crates.".to_string(),
        ],
    }
}

pub fn validate_manifest(manifest: &RlineManifest) -> Result<(), RlineError> {
    if manifest.schema_version != RLINE_MANIFEST_SCHEMA {
        return Err(RlineError::UnsupportedSchema(
            manifest.schema_version.clone(),
        ));
    }
    if manifest.family.trim().is_empty() {
        return Err(RlineError::EmptyFamily);
    }

    let mut crate_names = BTreeSet::new();
    for crate_spec in &manifest.crates {
        if crate_spec.name.trim().is_empty() {
            return Err(RlineError::EmptyCrateName);
        }
        if crate_spec.source_path.trim().is_empty() {
            return Err(RlineError::EmptySourcePath {
                crate_name: crate_spec.name.clone(),
            });
        }
        if !crate_names.insert(crate_spec.name.clone()) {
            return Err(RlineError::DuplicateCrateName {
                crate_name: crate_spec.name.clone(),
            });
        }
    }

    for crate_spec in &manifest.crates {
        for dependency in &crate_spec.internal_dependencies {
            if !crate_names.contains(dependency) {
                return Err(RlineError::UnknownInternalDependency {
                    crate_name: crate_spec.name.clone(),
                    dependency: dependency.clone(),
                });
            }
        }
    }

    for consumer in &manifest.consumers {
        if consumer.repo.trim().is_empty() {
            return Err(RlineError::EmptyConsumerRepo);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foundation_manifest_is_valid() {
        let manifest = foundation_manifest();

        validate_manifest(&manifest).unwrap();

        assert_eq!(manifest.schema_version, RLINE_MANIFEST_SCHEMA);
        assert!(manifest.crates.iter().any(|c| c.name == "rgraph-core"));
        assert!(manifest.consumers.iter().any(|c| c.repo == "BISECT"));
    }

    #[test]
    fn manifest_round_trips_through_json() {
        let manifest = foundation_manifest();

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let decoded: RlineManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, manifest);
    }

    #[test]
    fn duplicate_crates_are_rejected() {
        let mut manifest = foundation_manifest();
        manifest.crates.push(manifest.crates[0].clone());

        let error = validate_manifest(&manifest).unwrap_err();

        assert_eq!(
            error,
            RlineError::DuplicateCrateName {
                crate_name: "rctx-core".to_string()
            }
        );
    }

    #[test]
    fn unknown_internal_dependencies_are_rejected() {
        let mut manifest = foundation_manifest();
        manifest.crates[0]
            .internal_dependencies
            .push("missing-core".to_string());

        let error = validate_manifest(&manifest).unwrap_err();

        assert_eq!(
            error,
            RlineError::UnknownInternalDependency {
                crate_name: "rctx-core".to_string(),
                dependency: "missing-core".to_string()
            }
        );
    }
}
