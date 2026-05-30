//! Evidence sources. Each module fetches data from a different HuggingFace
//! endpoint and returns a [`FetchResult`] with source-specific evidence.

pub mod community;
pub mod hf_metadata;
pub mod model_config;
pub mod model_tree;

use crate::{BonaError, EvidenceSource, SourceRecord, SourceStatus};

/// Pull `base_model` out of HF cardData. May be a string or list of strings.
pub fn extract_base_model(card_data: &Option<serde_json::Value>) -> Option<String> {
    let cd = card_data.as_ref()?;
    let bm = cd.get("base_model")?;
    match bm {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(arr) => {
            arr.first().and_then(|v| v.as_str()).map(|s| s.to_string())
        }
        _ => None,
    }
}

/// The result of fetching a single evidence source.
pub struct FetchResult {
    pub record: SourceRecord,
    pub evidence: Option<Evidence>,
}

/// Source-specific evidence data, merged into the investigation after all
/// sources complete.
#[allow(dead_code)] // Variants constructed as sources are implemented.
pub enum Evidence {
    HfMetadata(hf_metadata::HfMetadataEvidence),
    ModelTree(model_tree::ModelTreeEvidence),
    ModelConfig(model_config::ModelConfigEvidence),
    Community(community::CommunityEvidence),
}

impl FetchResult {
    /// Convenience for a source that failed.
    pub fn failed(source: EvidenceSource, err: BonaError) -> Self {
        FetchResult {
            record: SourceRecord {
                source,
                status: SourceStatus::Failed {
                    reason: err.to_string(),
                },
            },
            evidence: None,
        }
    }

    /// Convenience for a source that succeeded.
    pub fn ok(source: EvidenceSource, fetched_ms: u64, evidence: Evidence) -> Self {
        FetchResult {
            record: SourceRecord {
                source,
                status: SourceStatus::Ok { fetched_ms },
            },
            evidence: Some(evidence),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_base_model_handles_string_and_list() {
        let s = serde_json::json!({ "base_model": "org/model" });
        assert_eq!(extract_base_model(&Some(s)), Some("org/model".to_string()));

        let l = serde_json::json!({ "base_model": ["org/model", "other"] });
        assert_eq!(extract_base_model(&Some(l)), Some("org/model".to_string()));

        assert_eq!(extract_base_model(&None), None);
    }
}
