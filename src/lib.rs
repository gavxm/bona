//! Bona engine library. CLI and web UI both consume the public API here.
//! [`ModelInvestigation`] is the stable contract - treat changes to it as
//! API changes.

use serde::{Deserialize, Serialize};

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
    Ok { fetched_ms: u64 },
    Failed { reason: String },
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

/// The investigation document. CLI prints it, web UI renders it, gallery
/// caches it as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInvestigation {
    pub schema_version: u32,
    pub model_id: String,
    pub declared: DeclaredFacts,
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
            sources: Vec::new(),
            findings: Vec::new(),
        }
    }

    /// Sort findings high-severity first.
    pub fn sort_findings(&mut self) {
        self.findings.sort_by(|a, b| b.severity.cmp(&a.severity));
    }
}

/// Subset of the HF `/api/models/{id}` response we actually use.
#[derive(Debug, Deserialize)]
struct HfModelInfo {
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    library_name: Option<String>,
    #[serde(default)]
    pipeline_tag: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    downloads: Option<u64>,
    /// Free-form card metadata. Base model lives in here (string or list).
    #[serde(rename = "cardData", default)]
    card_data: Option<serde_json::Value>,
}

/// Pull `base_model` out of cardData. May be a string or list of strings.
fn extract_base_model(card_data: &Option<serde_json::Value>) -> Option<String> {
    let cd = card_data.as_ref()?;
    let bm = cd.get("base_model")?;
    match bm {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(arr) => arr
            .first()
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

/// Fetch HF API metadata.
async fn fetch_hf_metadata(
    client: &reqwest::Client,
    model_id: &str,
    inv: &mut ModelInvestigation,
) {
    let start = std::time::Instant::now();
    let url = format!("https://huggingface.co/api/models/{model_id}");

    let result: Result<HfModelInfo, BonaError> = async {
        let resp = client.get(&url).send().await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(BonaError::ModelNotFound(model_id.to_string()));
        }
        let resp = resp.error_for_status()?;
        let info = resp
            .json::<HfModelInfo>()
            .await
            .map_err(|e| BonaError::Parse(e.to_string()))?;
        Ok(info)
    }
    .await;

    match result {
        Ok(info) => {
            inv.declared.declared_license = info.license;
            inv.declared.library = info.library_name;
            inv.declared.pipeline_tag = info.pipeline_tag;
            inv.declared.tags = info.tags;
            inv.declared.downloads = info.downloads;
            inv.declared.declared_base_model = extract_base_model(&info.card_data);

            inv.sources.push(SourceRecord {
                source: EvidenceSource::HfMetadata,
                status: SourceStatus::Ok {
                    fetched_ms: start.elapsed().as_millis() as u64,
                },
            });
        }
        Err(e) => {
            inv.sources.push(SourceRecord {
                source: EvidenceSource::HfMetadata,
                status: SourceStatus::Failed {
                    reason: e.to_string(),
                },
            });
        }
    }
}

/// Build an investigation for the given model id.
pub async fn investigate(model_id: &str) -> Result<ModelInvestigation, BonaError> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("bona/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let mut inv = ModelInvestigation::new(model_id);

    fetch_hf_metadata(&client, model_id, &mut inv).await;

    // TODO: Build out evidence sources
    for source in [
        EvidenceSource::ModelTree,
        EvidenceSource::ModelConfig,
        EvidenceSource::CommunitySignals,
    ] {
        inv.sources.push(SourceRecord {
            source,
            status: SourceStatus::NotImplemented,
        });
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

    #[test]
    fn extract_base_model_handles_string_and_list() {
        let s = serde_json::json!({ "base_model": "meta-llama/Llama-3.1-8B" });
        assert_eq!(
            extract_base_model(&Some(s)),
            Some("meta-llama/Llama-3.1-8B".to_string())
        );

        let l = serde_json::json!({ "base_model": ["meta-llama/Llama-3.1-8B", "other"] });
        assert_eq!(
            extract_base_model(&Some(l)),
            Some("meta-llama/Llama-3.1-8B".to_string())
        );

        assert_eq!(extract_base_model(&None), None);
    }
}
