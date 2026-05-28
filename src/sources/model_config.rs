use serde::{Deserialize, Serialize};

use crate::EvidenceSource;

use super::FetchResult;

/// Evidence extracted from config.json and safetensors header.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelConfigEvidence {
    pub architecture: Option<String>,
    pub hidden_size: Option<u64>,
    pub param_count: Option<u64>,
    pub tokenizer: Option<String>,
}

pub async fn fetch(_client: &reqwest::Client, _model_id: &str) -> FetchResult {
    FetchResult::not_implemented(EvidenceSource::ModelConfig)
}
