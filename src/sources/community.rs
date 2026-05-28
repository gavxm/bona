use serde::{Deserialize, Serialize};

use crate::EvidenceSource;

use super::FetchResult;

/// Evidence about the uploader and community activity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunityEvidence {
    pub uploader_account_created: Option<String>,
    pub uploader_model_count: Option<u64>,
    pub discussion_count: Option<u64>,
    pub pr_count: Option<u64>,
}

pub async fn fetch(_client: &reqwest::Client, _model_id: &str) -> FetchResult {
    FetchResult::not_implemented(EvidenceSource::CommunitySignals)
}
