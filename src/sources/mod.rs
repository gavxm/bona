//! Evidence sources. Each module fetches data from a different HuggingFace
//! endpoint and returns a [`FetchResult`] with source-specific evidence.

pub mod community;
pub mod hf_metadata;
pub mod model_config;
pub mod model_tree;

use serde::{Deserialize, Serialize};

use crate::{BonaError, EvidenceSource, SourceRecord, SourceStatus};

/// How a model relates to its base model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    Finetune,
    Quantization,
    Merge,
    Adapter,
    Unknown,
}

impl std::fmt::Display for RelationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationKind::Finetune => write!(f, "finetune"),
            RelationKind::Quantization => write!(f, "quantized"),
            RelationKind::Merge => write!(f, "merge"),
            RelationKind::Adapter => write!(f, "adapter"),
            RelationKind::Unknown => write!(f, "derived"),
        }
    }
}

/// A base model reference with its relation type.
#[derive(Debug, Clone)]
pub struct ParsedBaseModel {
    pub model_id: String,
    pub relation: RelationKind,
}

/// Parse a relation string (ex. "finetune", "quantized") into a RelationKind.
fn parse_relation(s: &str) -> RelationKind {
    match s.to_lowercase().as_str() {
        "finetune" | "fine-tune" => RelationKind::Finetune,
        "quantized" | "quantization" => RelationKind::Quantization,
        "merge" | "merged" => RelationKind::Merge,
        "adapter" => RelationKind::Adapter,
        _ => RelationKind::Unknown,
    }
}

/// Pull all base model references out of HF cardData.
///
/// Handles three formats:
/// - String: `"base_model": "org/model"`
/// - Array of strings: `"base_model": ["org/a", "org/b"]`
/// - Array of objects: `"base_model": [{"model": "org/a", "relation": "finetune"}]`
pub fn extract_base_models(card_data: &Option<serde_json::Value>) -> Vec<ParsedBaseModel> {
    let cd = match card_data.as_ref() {
        Some(cd) => cd,
        None => return Vec::new(),
    };
    let bm = match cd.get("base_model") {
        Some(bm) => bm,
        None => return Vec::new(),
    };
    match bm {
        serde_json::Value::String(s) => {
            vec![ParsedBaseModel {
                model_id: s.clone(),
                relation: RelationKind::Unknown,
            }]
        }
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| match v {
                serde_json::Value::String(s) => Some(ParsedBaseModel {
                    model_id: s.clone(),
                    relation: RelationKind::Unknown,
                }),
                serde_json::Value::Object(obj) => {
                    let model_id = obj.get("model")?.as_str()?.to_string();
                    let relation = obj
                        .get("relation")
                        .and_then(|r| r.as_str())
                        .map(parse_relation)
                        .unwrap_or(RelationKind::Unknown);
                    Some(ParsedBaseModel { model_id, relation })
                }
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Normalize the HF `gated` field (bool or string) to a string.
pub fn parse_gated(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
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
    fn extract_base_models_compat_with_old_string_and_list() {
        let s = serde_json::json!({ "base_model": "org/model" });
        let result = extract_base_models(&Some(s));
        assert_eq!(result[0].model_id, "org/model");

        let l = serde_json::json!({ "base_model": ["org/model", "other"] });
        let result = extract_base_models(&Some(l));
        assert_eq!(result[0].model_id, "org/model");
        assert_eq!(result.len(), 2);

        assert!(extract_base_models(&None).is_empty());
    }

    #[test]
    fn extract_base_models_string() {
        let cd = serde_json::json!({ "base_model": "org/model" });
        let result = extract_base_models(&Some(cd));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].model_id, "org/model");
        assert_eq!(result[0].relation, RelationKind::Unknown);
    }

    #[test]
    fn extract_base_models_array_of_strings() {
        let cd = serde_json::json!({ "base_model": ["org/a", "org/b"] });
        let result = extract_base_models(&Some(cd));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].model_id, "org/a");
        assert_eq!(result[1].model_id, "org/b");
        // Multiple base models with no relation implies merge.
        assert_eq!(result[0].relation, RelationKind::Unknown);
    }

    #[test]
    fn extract_base_models_array_of_objects() {
        let cd = serde_json::json!({
            "base_model": [
                { "model": "org/parent", "relation": "finetune" },
                { "model": "org/adapter-base", "relation": "adapter" },
            ]
        });
        let result = extract_base_models(&Some(cd));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].model_id, "org/parent");
        assert_eq!(result[0].relation, RelationKind::Finetune);
        assert_eq!(result[1].model_id, "org/adapter-base");
        assert_eq!(result[1].relation, RelationKind::Adapter);
    }

    #[test]
    fn extract_base_models_quantized_relation() {
        let cd = serde_json::json!({
            "base_model": [{ "model": "org/base", "relation": "quantized" }]
        });
        let result = extract_base_models(&Some(cd));
        assert_eq!(result[0].relation, RelationKind::Quantization);
    }

    #[test]
    fn extract_base_models_none() {
        assert!(extract_base_models(&None).is_empty());
        let cd = serde_json::json!({});
        assert!(extract_base_models(&Some(cd)).is_empty());
    }

    #[test]
    fn relation_kind_display() {
        assert_eq!(RelationKind::Finetune.to_string(), "finetune");
        assert_eq!(RelationKind::Quantization.to_string(), "quantized");
        assert_eq!(RelationKind::Merge.to_string(), "merge");
        assert_eq!(RelationKind::Adapter.to_string(), "adapter");
        assert_eq!(RelationKind::Unknown.to_string(), "derived");
    }
}
