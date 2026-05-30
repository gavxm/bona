//! Community signals source. Fetches uploader account info and discussion
//! counts from the HuggingFace API.

use serde::{Deserialize, Serialize};

use crate::{EvidenceSource, InvestigationError};

use super::{Evidence, FetchResult};

/// Evidence about the uploader and community activity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunityEvidence {
    /// The uploader (author) name.
    pub author: Option<String>,
    /// When the uploader account was created (ISO 8601).
    pub author_created_at: Option<String>,
    /// How many models the uploader has published.
    pub author_model_count: Option<u64>,
    /// Total discussion threads on this model.
    pub discussion_count: Option<u64>,
    /// How many of those discussions are closed/resolved.
    pub closed_discussion_count: Option<u64>,
}

/// Response from `/api/users/{name}/overview`.
#[derive(Debug, Deserialize)]
struct UserOverview {
    #[serde(rename = "createdAt", default)]
    created_at: Option<String>,
    #[serde(rename = "numModels", default)]
    num_models: Option<u64>,
}

/// Response from `/api/models/{id}/discussions`.
#[derive(Debug, Deserialize)]
struct DiscussionsResponse {
    #[serde(default)]
    count: Option<u64>,
    #[serde(rename = "numClosedDiscussions", default)]
    num_closed_discussions: Option<u64>,
}

/// Fetch community signals.
pub async fn fetch(
    client: &reqwest_middleware::ClientWithMiddleware,
    base_url: &str,
    model_id: &str,
    author: Option<&str>,
) -> FetchResult {
    let start = std::time::Instant::now();

    let mut evidence = CommunityEvidence::default();
    let mut any_succeeded = false;

    // Fan out: user overview (if we have an author) + discussions.
    let (user_result, discussions_result) = tokio::join!(
        async {
            match author {
                Some(name) => fetch_user_overview(client, base_url, name).await,
                None => Ok(None),
            }
        },
        fetch_discussions(client, base_url, model_id),
    );

    if let Some(name) = author {
        evidence.author = Some(name.to_string());
    }

    if let Ok(Some(ref overview)) = user_result {
        evidence.author_created_at = overview.created_at.clone();
        evidence.author_model_count = overview.num_models;
        any_succeeded = true;
    }

    if let Ok(ref discussions) = discussions_result {
        evidence.discussion_count = discussions.count;
        evidence.closed_discussion_count = discussions.num_closed_discussions;
        any_succeeded = true;
    }

    let ms = start.elapsed().as_millis() as u64;

    if any_succeeded {
        FetchResult::ok(
            EvidenceSource::CommunitySignals,
            ms,
            Evidence::Community(evidence),
        )
    } else {
        let reason = match (user_result, discussions_result) {
            (Err(e), _) => format!("user overview failed: {e}"),
            (_, Err(e)) => format!("discussions fetch failed: {e}"),
            _ => "no community data available".to_string(),
        };
        FetchResult::failed(
            EvidenceSource::CommunitySignals,
            InvestigationError::Parse(reason),
        )
    }
}

/// Fetch uploader account info from the user overview endpoint.
/// Returns None for orgs (the endpoint only works for individual users).
async fn fetch_user_overview(
    client: &reqwest_middleware::ClientWithMiddleware,
    base_url: &str,
    author: &str,
) -> Result<Option<UserOverview>, InvestigationError> {
    let url = format!("{base_url}/api/users/{author}/overview");
    let resp = client.get(&url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND
        || resp.status() == reqwest::StatusCode::UNAUTHORIZED
        || resp.status() == reqwest::StatusCode::FORBIDDEN
    {
        return Ok(None);
    }

    let resp = resp.error_for_status()?;
    let overview: UserOverview = resp
        .json()
        .await
        .map_err(|e| InvestigationError::Parse(e.to_string()))?;
    Ok(Some(overview))
}

/// Fetch discussion/PR counts for this model.
async fn fetch_discussions(
    client: &reqwest_middleware::ClientWithMiddleware,
    base_url: &str,
    model_id: &str,
) -> Result<DiscussionsResponse, InvestigationError> {
    let url = format!("{base_url}/api/models/{model_id}/discussions?limit=1");
    let resp = client.get(&url).send().await?;
    let resp = resp.error_for_status()?;
    let discussions: DiscussionsResponse = resp
        .json()
        .await
        .map_err(|e| InvestigationError::Parse(e.to_string()))?;
    Ok(discussions)
}
