use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FacilityError {
    #[error("facility id is empty")]
    EmptyFacilityId,
    #[error("facility {facility_id} has empty name")]
    EmptyFacilityName { facility_id: String },
    #[error("facility {facility_id} has no categories")]
    EmptyFacilityCategories { facility_id: String },
    #[error("facility {facility_id} has duplicate category: {category}")]
    DuplicateCategory {
        facility_id: String,
        category: String,
    },
    #[error("facility {facility_id} has duplicate capability: {capability_id}")]
    DuplicateCapability {
        facility_id: String,
        capability_id: String,
    },
    #[error("facility {facility_id} capability id is empty")]
    EmptyCapabilityId { facility_id: String },
    #[error("facility {facility_id} requirement id is empty")]
    EmptyRequirementId { facility_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilitySpec {
    pub facility_id: String,
    pub name: String,
    pub categories: Vec<FacilityCategory>,
    #[serde(default)]
    pub capabilities: Vec<FacilityCapability>,
    #[serde(default)]
    pub requirements: Vec<FacilityRequirement>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FacilityCategory {
    Production,
    Storage,
    Civic,
    Market,
    Transport,
    Utility,
    Housing,
    Defense,
    Cultural,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityCapability {
    pub capability_id: String,
    pub label: String,
    #[serde(default)]
    pub capacity_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityRequirement {
    pub requirement_id: String,
    pub label: String,
    pub requirement_type: RequirementType,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementType {
    Input,
    Labor,
    Access,
    Utility,
    Governance,
    Maintenance,
    Safety,
    Other,
}

pub fn validate_facility_spec(spec: &FacilitySpec) -> Result<(), FacilityError> {
    if spec.facility_id.trim().is_empty() {
        return Err(FacilityError::EmptyFacilityId);
    }
    if spec.name.trim().is_empty() {
        return Err(FacilityError::EmptyFacilityName {
            facility_id: spec.facility_id.clone(),
        });
    }
    if spec.categories.is_empty() {
        return Err(FacilityError::EmptyFacilityCategories {
            facility_id: spec.facility_id.clone(),
        });
    }

    let mut categories = BTreeSet::new();
    for category in &spec.categories {
        if !categories.insert(category) {
            return Err(FacilityError::DuplicateCategory {
                facility_id: spec.facility_id.clone(),
                category: format!("{category:?}"),
            });
        }
    }

    let mut capabilities = BTreeSet::new();
    for capability in &spec.capabilities {
        if capability.capability_id.trim().is_empty() {
            return Err(FacilityError::EmptyCapabilityId {
                facility_id: spec.facility_id.clone(),
            });
        }
        if !capabilities.insert(capability.capability_id.clone()) {
            return Err(FacilityError::DuplicateCapability {
                facility_id: spec.facility_id.clone(),
                capability_id: capability.capability_id.clone(),
            });
        }
    }

    for requirement in &spec.requirements {
        if requirement.requirement_id.trim().is_empty() {
            return Err(FacilityError::EmptyRequirementId {
                facility_id: spec.facility_id.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_facility() -> FacilitySpec {
        FacilitySpec {
            facility_id: "shared-forge".to_string(),
            name: "Shared Forge".to_string(),
            categories: vec![FacilityCategory::Production, FacilityCategory::Civic],
            capabilities: vec![FacilityCapability {
                capability_id: "metal-repair".to_string(),
                label: "Metal repair work".to_string(),
                capacity_note: Some("small-town repair throughput".to_string()),
            }],
            requirements: vec![FacilityRequirement {
                requirement_id: "wagon-access".to_string(),
                label: "Wagon access for fuel and stock".to_string(),
                requirement_type: RequirementType::Access,
                note: None,
            }],
            notes: vec![
                "Product-neutral facility primitive; placement belongs elsewhere.".to_string(),
            ],
        }
    }

    #[test]
    fn validates_product_neutral_facility_specs() {
        let facility = sample_facility();

        validate_facility_spec(&facility).unwrap();
        assert_eq!(facility.categories.len(), 2);
    }

    #[test]
    fn rejects_duplicate_capabilities() {
        let mut facility = sample_facility();
        facility.capabilities.push(facility.capabilities[0].clone());

        assert!(matches!(
            validate_facility_spec(&facility),
            Err(FacilityError::DuplicateCapability { .. })
        ));
    }

    #[test]
    fn round_trips_through_json() {
        let facility = sample_facility();
        let json = serde_json::to_string(&facility).unwrap();
        let decoded: FacilitySpec = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, facility);
    }
}
