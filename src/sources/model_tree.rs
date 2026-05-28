use serde::{Deserialize, Serialize};

use crate::{BonaError, EvidenceSource};

use super::{Evidence, FetchResult};

/// Evidence about the model's lineage relationships.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelTreeEvidence {
    /// The declared base/parent model id.
    pub parent_id: Option<String>,
    /// The parent model's license.
    pub parent_license: Option<String>,
    /// Whether the parent model id resolved (exists on HF).
    pub parent_exists: Option<bool>,
    /// Top sibling models, by downloads.
    pub siblings: Vec<String>,
}

/// Minimal response shape for the parent model lookup.
#[derive(Debug, Deserialize)]
struct HfModelInfo {
    #[serde(rename = "cardData", default)]
    card_data: Option<serde_json::Value>,
}

/// Minimal response shape for the sibling search.
#[derive(Debug, Deserialize)]
struct HfSearchResult {
    #[serde(default)]
    id: Option<String>,
}

/// Extract the license from cardData, falling back to license tag.
fn extract_license(card_data: &Option<serde_json::Value>) -> Option<String> {
    let cd = card_data.as_ref()?;
    cd.get("license")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Get the declared base model from the model's API response.
fn extract_base_model(card_data: &Option<serde_json::Value>) -> Option<String> {
    let cd = card_data.as_ref()?;
    let bm = cd.get("base_model")?;
    match bm {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(arr) => {
            arr.first().and_then(|v| v.as_str()).map(|s| s.to_string())
        }
        _ => None,
    }
}

pub async fn fetch(client: &reqwest::Client, model_id: &str) -> FetchResult {
    let start = std::time::Instant::now();

    let mut evidence = ModelTreeEvidence::default();

    // Step 1: get this model's declared base model.
    let parent_id = match fetch_base_model(client, model_id).await {
        Ok(id) => id,
        Err(e) => return FetchResult::failed(EvidenceSource::ModelTree, e),
    };

    let Some(parent_id) = parent_id else {
        // No declared parent - still a valid result.
        let ms = start.elapsed().as_millis() as u64;
        return FetchResult::ok(EvidenceSource::ModelTree, ms, Evidence::ModelTree(evidence));
    };

    evidence.parent_id = Some(parent_id.clone());

    // Step 2+3: fetch parent info and siblings concurrently.
    let (parent_result, siblings_result) = tokio::join!(
        fetch_parent_info(client, &parent_id),
        fetch_siblings(client, model_id, &parent_id),
    );

    match parent_result {
        Ok((exists, license)) => {
            evidence.parent_exists = Some(exists);
            evidence.parent_license = license;
        }
        Err(_) => {
            evidence.parent_exists = Some(false);
        }
    }

    if let Ok(siblings) = siblings_result {
        evidence.siblings = siblings;
    }

    let ms = start.elapsed().as_millis() as u64;
    FetchResult::ok(EvidenceSource::ModelTree, ms, Evidence::ModelTree(evidence))
}

/// Fetch this model's cardData to get the declared base model.
async fn fetch_base_model(
    client: &reqwest::Client,
    model_id: &str,
) -> Result<Option<String>, BonaError> {
    let url = format!("https://huggingface.co/api/models/{model_id}");
    let resp = client.get(&url).send().await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(BonaError::ModelNotFound(model_id.to_string()));
    }
    let resp = resp.error_for_status()?;
    let info: HfModelInfo = resp
        .json()
        .await
        .map_err(|e| BonaError::Parse(e.to_string()))?;
    Ok(extract_base_model(&info.card_data))
}

/// Fetch the parent model's metadata to get its license.
async fn fetch_parent_info(
    client: &reqwest::Client,
    parent_id: &str,
) -> Result<(bool, Option<String>), BonaError> {
    let url = format!("https://huggingface.co/api/models/{parent_id}");
    let resp = client.get(&url).send().await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok((false, None));
    }
    let resp = resp.error_for_status()?;
    let info: HfModelInfo = resp
        .json()
        .await
        .map_err(|e| BonaError::Parse(e.to_string()))?;
    Ok((true, extract_license(&info.card_data)))
}

/// Find sibling models (other finetunings of the same parent).
async fn fetch_siblings(
    client: &reqwest::Client,
    model_id: &str,
    parent_id: &str,
) -> Result<Vec<String>, BonaError> {
    let url = format!(
        "https://huggingface.co/api/models?filter=base_model:{parent_id}&sort=downloads&direction=-1&limit=6"
    );
    let resp = client.get(&url).send().await?;
    let resp = resp.error_for_status()?;
    let results: Vec<HfSearchResult> = resp
        .json()
        .await
        .map_err(|e| BonaError::Parse(e.to_string()))?;

    Ok(results
        .into_iter()
        .filter_map(|r| r.id)
        .filter(|id| id != model_id)
        .take(5)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_base_model_handles_variants() {
        let s = serde_json::json!({ "base_model": "org/model" });
        assert_eq!(extract_base_model(&Some(s)), Some("org/model".to_string()));

        let l = serde_json::json!({ "base_model": ["org/model", "other"] });
        assert_eq!(extract_base_model(&Some(l)), Some("org/model".to_string()));

        assert_eq!(extract_base_model(&None), None);
    }

    #[test]
    fn extract_license_from_card_data() {
        let cd = serde_json::json!({ "license": "mit" });
        assert_eq!(extract_license(&Some(cd)), Some("mit".to_string()));

        let cd = serde_json::json!({ "other": "field" });
        assert_eq!(extract_license(&Some(cd)), None);

        assert_eq!(extract_license(&None), None);
    }
}
