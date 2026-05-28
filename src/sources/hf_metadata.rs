use serde::Deserialize;

use crate::{BonaError, DeclaredFacts, EvidenceSource};

use super::{Evidence, FetchResult};

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

pub async fn fetch(client: &reqwest::Client, model_id: &str) -> FetchResult {
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

#[cfg(test)]
mod tests {
    use super::*;

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
