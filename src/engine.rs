//! Investigation orchestration. Builds an HTTP client, fetches evidence from
//! all sources, and runs the findings engine.

use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

use crate::sources::{Evidence, RelationKind};
use crate::{InvestigationError, ModelInvestigation, findings, sources};

const HF_BASE_URL: &str = "https://huggingface.co";

/// Build an investigation for the given model id.
pub async fn investigate(model_id: &str) -> Result<ModelInvestigation, InvestigationError> {
    investigate_with_base_url(model_id, HF_BASE_URL).await
}

#[doc(hidden)]
pub async fn investigate_with_base_url(
    model_id: &str,
    base_url: &str,
) -> Result<ModelInvestigation, InvestigationError> {
    let mut builder =
        reqwest::Client::builder().user_agent(concat!("bona/", env!("CARGO_PKG_VERSION")));

    if let Ok(token) = std::env::var("HF_TOKEN") {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|e| InvestigationError::Parse(format!("invalid HF_TOKEN: {e}")))?,
        );
        builder = builder.default_headers(headers);
    }

    let raw_client = builder.build()?;

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(2);
    let client = ClientBuilder::new(raw_client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let mut inv = ModelInvestigation::new(model_id);

    // Phase 1: fetch HF metadata (other sources depend on its results).
    let hf = sources::hf_metadata::fetch(&client, base_url, model_id).await;
    inv.sources.push(hf.record);
    let (base_model, relation) = if let Some(Evidence::HfMetadata(e)) = hf.evidence {
        let bm = e.declared.declared_base_model.clone();
        let rel = e
            .declared
            .base_model_relation
            .unwrap_or(RelationKind::Unknown);
        inv.declared = e.declared;
        (bm, rel)
    } else {
        (None, RelationKind::Unknown)
    };

    // Phase 2: remaining sources fan out concurrently.
    let author = model_id.split_once('/').map(|(org, _)| org);
    let (tree, config, community) = tokio::join!(
        sources::model_tree::fetch(&client, base_url, model_id, base_model.as_deref(), relation),
        sources::model_config::fetch(&client, base_url, model_id),
        sources::community::fetch(&client, base_url, model_id, author),
    );

    for result in [tree, config, community] {
        inv.sources.push(result.record);
        if let Some(evidence) = result.evidence {
            match evidence {
                Evidence::HfMetadata(_) => {}
                Evidence::ModelTree(e) => inv.lineage = Some(e),
                Evidence::ModelConfig(e) => inv.config = Some(e),
                Evidence::Community(e) => inv.community = Some(e),
            }
        }
    }

    findings::compute(&mut inv);
    inv.sort_findings();

    Ok(inv)
}
