//! Investigation orchestration. Builds an HTTP client, fetches evidence from
//! all sources, and runs the findings engine.

use crate::sources::Evidence;
use crate::{BonaError, ModelInvestigation, findings, sources};

const HF_BASE_URL: &str = "https://huggingface.co";

/// Build an investigation for the given model id.
pub async fn investigate(model_id: &str) -> Result<ModelInvestigation, BonaError> {
    investigate_with_base_url(model_id, HF_BASE_URL).await
}

#[doc(hidden)]
pub async fn investigate_with_base_url(
    model_id: &str,
    base_url: &str,
) -> Result<ModelInvestigation, BonaError> {
    let mut builder =
        reqwest::Client::builder().user_agent(concat!("bona/", env!("CARGO_PKG_VERSION")));

    if let Ok(token) = std::env::var("HF_TOKEN") {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|e| BonaError::Parse(format!("invalid HF_TOKEN: {e}")))?,
        );
        builder = builder.default_headers(headers);
    }

    let client = builder.build()?;
    let mut inv = ModelInvestigation::new(model_id);

    // Phase 1: fetch HF metadata (other sources depend on its results).
    let hf = sources::hf_metadata::fetch(&client, base_url, model_id).await;
    inv.sources.push(hf.record);
    let base_model = if let Some(Evidence::HfMetadata(e)) = hf.evidence {
        let bm = e.declared.declared_base_model.clone();
        inv.declared = e.declared;
        bm
    } else {
        None
    };

    // Phase 2: remaining sources fan out concurrently.
    let author = model_id.split_once('/').map(|(org, _)| org);
    let (tree, config, community) = tokio::join!(
        sources::model_tree::fetch(&client, base_url, model_id, base_model.as_deref()),
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
