//! Bona engine library. CLI and web UI both consume the public API here.
//! [`ModelInvestigation`] is the stable contract - treat changes to it as
//! API changes.

mod sources;

use serde::{Deserialize, Serialize};
use sources::Evidence;

/// Engine errors.
#[derive(Debug, thiserror::Error)]
pub enum BonaError {
    #[error("network/HTTP error talking to HuggingFace: {0}")]
    Http(#[from] reqwest::Error),

    #[error("model not found on HuggingFace: {0}")]
    ModelNotFound(String),

    #[error("failed to parse a response: {0}")]
    Parse(String),
}

/// Bump when [`ModelInvestigation`] changes in a breaking way.
pub const SCHEMA_VERSION: u32 = 1;

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
    pub library: Option<String>,
    pub pipeline_tag: Option<String>,
    pub tags: Vec<String>,
    pub downloads: Option<u64>,
}

pub use sources::community::CommunityEvidence;
pub use sources::model_config::ModelConfigEvidence;
pub use sources::model_tree::ModelTreeEvidence;

/// The investigation document. CLI prints it, web UI renders it, gallery
/// caches it as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInvestigation {
    pub schema_version: u32,
    pub model_id: String,
    pub declared: DeclaredFacts,
    pub lineage: Option<ModelTreeEvidence>,
    pub config: Option<ModelConfigEvidence>,
    pub community: Option<CommunityEvidence>,
    pub sources: Vec<SourceRecord>,
    pub findings: Vec<Finding>,
}

impl ModelInvestigation {
    fn new(model_id: &str) -> Self {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            model_id: model_id.to_string(),
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

/// Build an investigation for the given model id.
pub async fn investigate(model_id: &str) -> Result<ModelInvestigation, BonaError> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("bona/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let mut inv = ModelInvestigation::new(model_id);

    let (hf, tree, config, community) = tokio::join!(
        sources::hf_metadata::fetch(&client, model_id),
        sources::model_tree::fetch(&client, model_id),
        sources::model_config::fetch(&client, model_id),
        sources::community::fetch(&client, model_id),
    );

    for result in [hf, tree, config, community] {
        inv.sources.push(result.record);
        if let Some(evidence) = result.evidence {
            match evidence {
                Evidence::HfMetadata(e) => inv.declared = e.declared,
                Evidence::ModelTree(e) => inv.lineage = Some(e),
                Evidence::ModelConfig(e) => inv.config = Some(e),
                Evidence::Community(e) => inv.community = Some(e),
            }
        }
    }

    compute_findings(&mut inv);
    inv.sort_findings();

    Ok(inv)
}

/// Compute findings from gathered evidence.
fn compute_findings(_inv: &mut ModelInvestigation) {
    // TODO: lineage inconsistency, license inheritance violation,
    // documentation gap, trust signals, metadata anomaly.
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
            evidence_url: None,
        });
        inv.findings.push(Finding {
            id: "b".into(),
            title: "high one".into(),
            severity: Severity::High,
            detail: "".into(),
            evidence_url: None,
        });
        inv.sort_findings();
        assert_eq!(inv.findings[0].severity, Severity::High);
    }
}
