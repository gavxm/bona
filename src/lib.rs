//! yurai engine library. CLI and web UI both consume the public API here.
//! [`ModelInvestigation`] is the stable contract - treat changes to it as
//! API changes.

mod engine;
mod findings;
pub mod output;
mod sources;

use serde::{Deserialize, Serialize};

// Re-export the engine entry points.
pub use engine::build_client;
pub use engine::investigate;
pub use engine::investigate_with_base_url;
pub use engine::investigate_with_client;

/// Engine errors.
#[derive(Debug, thiserror::Error)]
pub enum InvestigationError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP middleware error: {0}")]
    Middleware(#[from] reqwest_middleware::Error),

    #[error("model not found on HuggingFace: {0}")]
    ModelNotFound(String),

    #[error("failed to parse a response: {0}")]
    Parse(String),
}

/// Bump when [`ModelInvestigation`] changes in a breaking way.
pub const SCHEMA_VERSION: u32 = 3;

/// Weight file extensions recognized in model repos.
pub const WEIGHT_EXTENSIONS: &[&str] = &[
    ".safetensors",
    ".bin",
    ".pt",
    ".pth",
    ".gguf",
    ".ggml",
    ".onnx",
    ".tflite",
    ".h5",
    ".msgpack",
];

/// Ordered low-to-high so findings can be sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
}

/// Which evidence source a piece of evidence came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    /// HF API metadata (card, license, declared base model, tags).
    HfMetadata,
    /// Model tree (parent/child/sibling relationships).
    ModelTree,
    /// config.json + safetensors header (architecture, params).
    ModelConfig,
    /// Community signals (uploader account, discussions).
    CommunitySignals,
}

/// Whether an evidence source was successfully fetched.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SourceStatus {
    Ok {
        fetched_ms: u64,
    },
    Failed {
        reason: String,
    },
    /// Not yet implemented.
    NotImplemented,
}

/// A fetched evidence source and its status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub source: EvidenceSource,
    pub status: SourceStatus,
}

/// A single finding. Each one cites its evidence so the UI can link back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Machine id, ex. "license_inheritance_violation".
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub detail: String,
    /// Why this severity was assigned; helps users triage.
    pub reason: String,
    /// The declared value that triggered this finding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_value: Option<String>,
    /// The actual/observed value that contradicts the declared value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_value: Option<String>,
    /// Link to the underlying evidence on HF.
    pub evidence_url: Option<String>,
}

/// What the model declares about itself (card/metadata). Kept separate from
/// derived facts to spot contradictions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeclaredFacts {
    pub model_id: String,
    pub declared_license: Option<String>,
    pub declared_base_model: Option<String>,
    /// How this model relates to its base model (finetune, quantized, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_model_relation: Option<sources::RelationKind>,
    pub library: Option<String>,
    pub pipeline_tag: Option<String>,
    pub tags: Vec<String>,
    pub downloads: Option<u64>,
    /// Access control status: "false", "auto", or "manual".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gated: Option<String>,
    /// Latest commit hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// ISO 8601 timestamp of last modification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    /// Filenames in the model repo.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    /// ISO 8601 timestamp of model creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

pub use sources::RelationKind;
pub use sources::community::CommunityEvidence;
pub use sources::model_config::ModelConfigEvidence;
pub use sources::model_tree::{LineageEvidence, LineageNode, MAX_LINEAGE_DEPTH};

/// The investigation document. CLI prints it, web UI renders it, gallery
/// caches it as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInvestigation {
    pub schema_version: u32,
    pub model_id: String,
    pub investigated_at: String,
    pub declared: DeclaredFacts,
    pub lineage: Option<LineageEvidence>,
    pub config: Option<ModelConfigEvidence>,
    pub community: Option<CommunityEvidence>,
    pub sources: Vec<SourceRecord>,
    pub findings: Vec<Finding>,
}

impl ModelInvestigation {
    pub(crate) fn new(model_id: &str) -> Self {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            model_id: model_id.to_string(),
            investigated_at: chrono::Utc::now().to_rfc3339(),
            declared: DeclaredFacts {
                model_id: model_id.to_string(),
                ..Default::default()
            },
            lineage: None,
            config: None,
            community: None,
            sources: Vec::new(),
            findings: Vec::new(),
        }
    }

    /// Sort findings high-severity first.
    pub fn sort_findings(&mut self) {
        self.findings.sort_by_key(|f| std::cmp::Reverse(f.severity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_investigation_carries_model_id_and_schema() {
        let inv = ModelInvestigation::new("meta-llama/Llama-3.1-8B-Instruct");
        assert_eq!(inv.model_id, "meta-llama/Llama-3.1-8B-Instruct");
        assert_eq!(inv.schema_version, SCHEMA_VERSION);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn findings_sort_high_severity_first() {
        let mut inv = ModelInvestigation::new("x/y");
        inv.findings.push(Finding {
            id: "a".into(),
            title: "low one".into(),
            severity: Severity::Low,
            detail: "".into(),
            reason: "".into(),
            declared_value: None,
            actual_value: None,
            evidence_url: None,
        });
        inv.findings.push(Finding {
            id: "b".into(),
            title: "high one".into(),
            severity: Severity::High,
            detail: "".into(),
            reason: "".into(),
            declared_value: None,
            actual_value: None,
            evidence_url: None,
        });
        inv.sort_findings();
        assert_eq!(inv.findings[0].severity, Severity::High);
    }
}
