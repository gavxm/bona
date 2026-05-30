//! Model tree source. Fetches parent model metadata, license, and sibling
//! models from the HuggingFace API.

use serde::{Deserialize, Serialize};

use crate::{BonaError, EvidenceSource};

use super::{Evidence, FetchResult, RelationKind};

/// Maximum ancestor depth to walk.
pub const MAX_LINEAGE_DEPTH: u32 = 4;

/// A single node in the lineage chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    pub model_id: String,
    pub license: Option<String>,
    pub relation: RelationKind,
    pub exists: bool,
    pub gated: Option<String>,
    /// 0-indexed depth in the chain: 0 = direct parent, 1 = grandparent, etc.
    /// Display as `depth + 1` for user-facing "depth N" labels.
    pub depth: u32,
}

/// Multi-hop lineage evidence.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineageEvidence {
    /// Ancestor chain, ordered from direct parent to most distant ancestor.
    pub chain: Vec<LineageNode>,
    /// Top sibling models (other derivatives of the same direct parent).
    pub siblings: Vec<String>,
}

impl LineageEvidence {
    /// Direct parent model id, if any.
    pub fn parent_id(&self) -> Option<&str> {
        self.chain.first().map(|n| n.model_id.as_str())
    }

    /// Direct parent license, if any.
    pub fn parent_license(&self) -> Option<&str> {
        self.chain.first().and_then(|n| n.license.as_deref())
    }

    /// Whether the direct parent exists on HF.
    pub fn parent_exists(&self) -> Option<bool> {
        self.chain.first().map(|n| n.exists)
    }

    /// Direct parent gated status, if any.
    pub fn parent_gated(&self) -> Option<&str> {
        self.chain.first().and_then(|n| n.gated.as_deref())
    }
}

/// Minimal response shape for the parent model lookup.
#[derive(Debug, Deserialize)]
struct HfModelInfo {
    #[serde(rename = "cardData", default)]
    card_data: Option<serde_json::Value>,
    #[serde(default)]
    gated: Option<serde_json::Value>,
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

/// Fetch lineage evidence: walk the base model chain up to MAX_LINEAGE_DEPTH
/// ancestors, fetching siblings only for the direct parent.
pub async fn fetch(
    client: &reqwest::Client,
    base_url: &str,
    model_id: &str,
    declared_base_model: Option<&str>,
    relation: RelationKind,
) -> FetchResult {
    let start = std::time::Instant::now();

    let mut evidence = LineageEvidence::default();

    let Some(parent_id) = declared_base_model else {
        let ms = start.elapsed().as_millis() as u64;
        return FetchResult::ok(EvidenceSource::ModelTree, ms, Evidence::ModelTree(evidence));
    };

    // First hop: fetch parent info and siblings concurrently.
    let (parent_result, siblings_result) = tokio::join!(
        fetch_parent_info(client, base_url, parent_id),
        fetch_siblings(client, base_url, model_id, parent_id),
    );

    let mut next_card_data: Option<serde_json::Value> = None;

    match parent_result {
        Ok(info) => {
            evidence.chain.push(LineageNode {
                model_id: parent_id.to_string(),
                license: info.license,
                relation,
                exists: info.exists,
                gated: info.gated,
                depth: 0,
            });
            next_card_data = info.card_data;
        }
        Err(_) => {
            evidence.chain.push(LineageNode {
                model_id: parent_id.to_string(),
                license: None,
                relation,
                exists: false,
                gated: None,
                depth: 0,
            });
        }
    }

    if let Ok(siblings) = siblings_result {
        evidence.siblings = siblings;
    }

    // Walk subsequent hops sequentially.
    let mut seen = std::collections::HashSet::new();
    seen.insert(model_id.to_string());
    seen.insert(parent_id.to_string());

    // Each hop depends on the previous response's base model, so these are
    // inherently sequential. MAX_LINEAGE_DEPTH caps total API calls.
    for depth in 1..MAX_LINEAGE_DEPTH {
        let ancestors = super::extract_base_models(&next_card_data);

        let ancestor = match ancestors.first() {
            Some(a) => a,
            None => break,
        };

        if seen.contains(&ancestor.model_id) {
            break; // Cycle detected.
        }
        seen.insert(ancestor.model_id.clone());

        match fetch_parent_info(client, base_url, &ancestor.model_id).await {
            Ok(info) => {
                evidence.chain.push(LineageNode {
                    model_id: ancestor.model_id.clone(),
                    license: info.license,
                    relation: ancestor.relation,
                    exists: info.exists,
                    gated: info.gated,
                    depth,
                });
                next_card_data = info.card_data;
            }
            Err(_) => {
                evidence.chain.push(LineageNode {
                    model_id: ancestor.model_id.clone(),
                    license: None,
                    relation: ancestor.relation,
                    exists: false,
                    gated: None,
                    depth,
                });
                break;
            }
        }
    }

    let ms = start.elapsed().as_millis() as u64;
    FetchResult::ok(EvidenceSource::ModelTree, ms, Evidence::ModelTree(evidence))
}

struct ParentInfo {
    exists: bool,
    license: Option<String>,
    gated: Option<String>,
    card_data: Option<serde_json::Value>,
}

/// Fetch the parent model's metadata.
async fn fetch_parent_info(
    client: &reqwest::Client,
    base_url: &str,
    parent_id: &str,
) -> Result<ParentInfo, BonaError> {
    let url = format!("{base_url}/api/models/{parent_id}");
    let resp = client.get(&url).send().await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(ParentInfo {
            exists: false,
            license: None,
            gated: None,
            card_data: None,
        });
    }
    let resp = resp.error_for_status()?;
    let info: HfModelInfo = resp
        .json()
        .await
        .map_err(|e| BonaError::Parse(e.to_string()))?;
    let gated = info.gated.as_ref().and_then(super::parse_gated);
    let license = extract_license(&info.card_data);
    Ok(ParentInfo {
        exists: true,
        license,
        gated,
        card_data: info.card_data,
    })
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
