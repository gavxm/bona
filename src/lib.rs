//! Bona engine library. CLI and web UI both consume the public API here.
//! [`ModelInvestigation`] is the stable contract - treat changes to it as
//! API changes.

mod findings;
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
    pub investigated_at: String,
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

const HF_BASE_URL: &str = "https://huggingface.co";

/// Build an investigation for the given model id.
pub async fn investigate(model_id: &str) -> Result<ModelInvestigation, BonaError> {
    investigate_with_base_url(model_id, HF_BASE_URL).await
}

#[doc(hidden)]
pub async fn investigate_with_base_url(
    model_id: &str,
    base_url: &str,
) -> Result<ModelInvestigation, BonaError> {
    let mut builder =
        reqwest::Client::builder().user_agent(concat!("bona/", env!("CARGO_PKG_VERSION")));

    if let Ok(token) = std::env::var("HF_TOKEN") {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|e| BonaError::Parse(format!("invalid HF_TOKEN: {e}")))?,
        );
        builder = builder.default_headers(headers);
    }

    let client = builder.build()?;
    let mut inv = ModelInvestigation::new(model_id);

    // Phase 1: fetch HF metadata (other sources depend on its results).
    let hf = sources::hf_metadata::fetch(&client, base_url, model_id).await;
    inv.sources.push(hf.record);
    let base_model = if let Some(Evidence::HfMetadata(e)) = hf.evidence {
        let bm = e.declared.declared_base_model.clone();
        inv.declared = e.declared;
        bm
    } else {
        None
    };

    // Phase 2: remaining sources fan out concurrently.
    let author = model_id.split_once('/').map(|(org, _)| org);
    let (tree, config, community) = tokio::join!(
        sources::model_tree::fetch(&client, base_url, model_id, base_model.as_deref()),
        sources::model_config::fetch(&client, base_url, model_id),
        sources::community::fetch(&client, base_url, model_id, author),
    );

    for result in [tree, config, community] {
        inv.sources.push(result.record);
        if let Some(evidence) = result.evidence {
            match evidence {
                Evidence::HfMetadata(_) => {}
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
fn compute_findings(inv: &mut ModelInvestigation) {
    findings::compute(inv);
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
