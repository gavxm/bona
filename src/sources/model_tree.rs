//! Model tree source. Fetches parent model metadata, license, and sibling
//! models from the HuggingFace API.

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

/// Extract the license from cardData.
fn extract_license(card_data: &Option<serde_json::Value>) -> Option<String> {
    let cd = card_data.as_ref()?;
    cd.get("license")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Fetch model tree evidence.
pub async fn fetch(
    client: &reqwest::Client,
    base_url: &str,
    model_id: &str,
    declared_base_model: Option<&str>,
) -> FetchResult {
    let start = std::time::Instant::now();

    let mut evidence = ModelTreeEvidence::default();

    let Some(parent_id) = declared_base_model else {
        // No declared parent - still a valid result.
        let ms = start.elapsed().as_millis() as u64;
        return FetchResult::ok(EvidenceSource::ModelTree, ms, Evidence::ModelTree(evidence));
    };

    evidence.parent_id = Some(parent_id.to_string());

    // Fetch parent info and siblings concurrently.
    let (parent_result, siblings_result) = tokio::join!(
        fetch_parent_info(client, base_url, parent_id),
        fetch_siblings(client, base_url, model_id, parent_id),
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

/// Fetch the parent model's metadata to get its license.
async fn fetch_parent_info(
    client: &reqwest::Client,
    base_url: &str,
    parent_id: &str,
) -> Result<(bool, Option<String>), BonaError> {
    let url = format!("{base_url}/api/models/{parent_id}");
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
    base_url: &str,
    model_id: &str,
    parent_id: &str,
) -> Result<Vec<String>, BonaError> {
    let url = format!(
        "{base_url}/api/models?filter=base_model:{parent_id}&sort=downloads&direction=-1&limit=6"
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
    fn extract_license_from_card_data() {
        let cd = serde_json::json!({ "license": "mit" });
        assert_eq!(extract_license(&Some(cd)), Some("mit".to_string()));

        let cd = serde_json::json!({ "other": "field" });
        assert_eq!(extract_license(&Some(cd)), None);

        assert_eq!(extract_license(&None), None);
    }
}
