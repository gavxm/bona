//! HF API metadata source. Fetches the model card, license, base model,
//! tags, and download count from `/api/models/{id}`.

use serde::Deserialize;

use crate::{DeclaredFacts, EvidenceSource, InvestigationError};

use super::{Evidence, FetchResult, extract_base_models, parse_gated};

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
    /// Access control: false, "auto", or "manual".
    #[serde(default)]
    gated: Option<serde_json::Value>,
    #[serde(default)]
    sha: Option<String>,
    #[serde(rename = "lastModified", default)]
    last_modified: Option<String>,
    #[serde(default)]
    likes: Option<u64>,
    #[serde(default)]
    private: Option<bool>,
    /// File listing. Each entry has an `rfilename` with the relative path.
    #[serde(default)]
    siblings: Option<Vec<HfSibling>>,
    #[serde(rename = "createdAt", default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HfSibling {
    rfilename: String,
}

pub async fn fetch(
    client: &reqwest_middleware::ClientWithMiddleware,
    base_url: &str,
    model_id: &str,
) -> FetchResult {
    let start = std::time::Instant::now();
    let url = format!("{base_url}/api/models/{model_id}");

    let result: Result<HfModelInfo, InvestigationError> = async {
        let resp = client.get(&url).send().await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(InvestigationError::ModelNotFound(model_id.to_string()));
        }
        let resp = resp.error_for_status()?;
        let info = resp
            .json::<HfModelInfo>()
            .await
            .map_err(|e| InvestigationError::Parse(e.to_string()))?;
        Ok(info)
    }
    .await;

    match result {
        Ok(info) => {
            let card_license = info
                .card_data
                .as_ref()
                .and_then(|cd| cd.get("license"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let gated = info.gated.as_ref().and_then(parse_gated);
            let files: Vec<String> = info
                .siblings
                .unwrap_or_default()
                .into_iter()
                .map(|s| s.rfilename)
                .collect();
            let base_models = extract_base_models(&info.card_data);
            let declared_base_model = base_models.first().map(|p| p.model_id.clone());
            let base_model_relation = base_models.first().map(|p| p.relation);
            let declared = DeclaredFacts {
                model_id: model_id.to_string(),
                declared_license: card_license.or(info.license),
                declared_base_model,
                base_model_relation,
                library: info.library_name,
                pipeline_tag: info.pipeline_tag,
                tags: info.tags,
                downloads: info.downloads,
                gated,
                sha: info.sha,
                last_modified: info.last_modified,
                likes: info.likes,
                private: info.private,
                files,
                created_at: info.created_at,
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
