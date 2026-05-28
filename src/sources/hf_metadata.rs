use serde::Deserialize;

use crate::{BonaError, DeclaredFacts, EvidenceSource};

use super::{Evidence, FetchResult, extract_base_model};

/// Evidence extracted from the HF API metadata endpoint.
pub struct HfMetadataEvidence {
    pub declared: DeclaredFacts,
}

/// Subset of the HF `/api/models/{id}` response.
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

pub async fn fetch(client: &reqwest::Client, base_url: &str, model_id: &str) -> FetchResult {
    let start = std::time::Instant::now();
    let url = format!("{base_url}/api/models/{model_id}");

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
            let declared = DeclaredFacts {
                model_id: model_id.to_string(),
                declared_license: info.license,
                declared_base_model: extract_base_model(&info.card_data),
                library: info.library_name,
                pipeline_tag: info.pipeline_tag,
                tags: info.tags,
                downloads: info.downloads,
            };
            let ms = start.elapsed().as_millis() as u64;
            FetchResult::ok(
                EvidenceSource::HfMetadata,
                ms,
                Evidence::HfMetadata(HfMetadataEvidence { declared }),
            )
        }
        Err(e) => FetchResult::failed(EvidenceSource::HfMetadata, e),
    }
}
