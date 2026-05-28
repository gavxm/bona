use serde::{Deserialize, Serialize};

use crate::EvidenceSource;

use super::FetchResult;

/// Evidence about the model's lineage relationships.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelTreeEvidence {
    pub parent: Option<String>,
    pub siblings: Vec<String>,
}

pub async fn fetch(_client: &reqwest::Client, _model_id: &str) -> FetchResult {
    FetchResult::not_implemented(EvidenceSource::ModelTree)
}
