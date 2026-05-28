use serde::{Deserialize, Serialize};

use crate::{BonaError, EvidenceSource};

use super::{Evidence, FetchResult};

/// Evidence extracted from config.json and safetensors header.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelConfigEvidence {
    /// Model architecture(s) from config.json (ex. "LlamaForCausalLM").
    pub architectures: Vec<String>,
    /// The `model_type` field from config.json (ex. "llama", "phi").
    pub model_type: Option<String>,
    pub hidden_size: Option<u64>,
    pub vocab_size: Option<u64>,
    pub num_hidden_layers: Option<u64>,
    /// Total weight size in bytes from safetensors index metadata.
    pub safetensors_total_size: Option<u64>,
    /// Tokenizer class from tokenizer_config.json.
    pub tokenizer_class: Option<String>,
}

/// Subset of config.json.
#[derive(Debug, Deserialize)]
struct ConfigJson {
    #[serde(default)]
    architectures: Option<Vec<String>>,
    #[serde(default)]
    model_type: Option<String>,
    #[serde(default)]
    hidden_size: Option<u64>,
    #[serde(default)]
    vocab_size: Option<u64>,
    #[serde(default)]
    num_hidden_layers: Option<u64>,
}

/// The safetensors index file metadata.
#[derive(Debug, Deserialize)]
struct SafetensorsIndex {
    #[serde(default)]
    metadata: Option<SafetensorsMetadata>,
}

#[derive(Debug, Deserialize)]
struct SafetensorsMetadata {
    #[serde(default)]
    total_size: Option<u64>,
}

/// Subset of tokenizer_config.json.
#[derive(Debug, Deserialize)]
struct TokenizerConfig {
    #[serde(default)]
    tokenizer_class: Option<String>,
}

pub async fn fetch(client: &reqwest::Client, base_url: &str, model_id: &str) -> FetchResult {
    let start = std::time::Instant::now();

    let mut evidence = ModelConfigEvidence::default();

    let (config_result, safetensors_result, tokenizer_result) = tokio::join!(
        fetch_config_json(client, base_url, model_id),
        fetch_safetensors_index(client, base_url, model_id),
        fetch_tokenizer_config(client, base_url, model_id),
    );

    let mut any_succeeded = false;

    if let Ok(Some(ref config)) = config_result {
        evidence.architectures = config.architectures.clone().unwrap_or_default();
        evidence.model_type = config.model_type.clone();
        evidence.hidden_size = config.hidden_size;
        evidence.vocab_size = config.vocab_size;
        evidence.num_hidden_layers = config.num_hidden_layers;
        any_succeeded = true;
    }

    if let Ok(Some(total_size)) = safetensors_result {
        evidence.safetensors_total_size = Some(total_size);
        any_succeeded = true;
    }

    if let Ok(Some(tokenizer_class)) = tokenizer_result {
        evidence.tokenizer_class = Some(tokenizer_class);
        any_succeeded = true;
    }

    let ms = start.elapsed().as_millis() as u64;

    if any_succeeded {
        FetchResult::ok(
            EvidenceSource::ModelConfig,
            ms,
            Evidence::ModelConfig(evidence),
        )
    } else {
        // All three failed or returned nothing.
        let reason = match config_result {
            Err(e) => format!("config.json fetch failed: {e}"),
            _ => "config.json not found or inaccessible".to_string(),
        };
        FetchResult::failed(
            EvidenceSource::ModelConfig,
            BonaError::Parse(reason),
        )
    }
}

/// Fetch and parse config.json from the model repo.
async fn fetch_config_json(
    client: &reqwest::Client,
    base_url: &str,
    model_id: &str,
) -> Result<Option<ConfigJson>, BonaError> {
    let url = format!("{base_url}/{model_id}/resolve/main/config.json");
    let resp = client.get(&url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND
        || resp.status() == reqwest::StatusCode::UNAUTHORIZED
        || resp.status() == reqwest::StatusCode::FORBIDDEN
    {
        return Ok(None);
    }

    let resp = resp.error_for_status()?;
    let config: ConfigJson = resp
        .json()
        .await
        .map_err(|e| BonaError::Parse(e.to_string()))?;
    Ok(Some(config))
}

/// Fetch total size from model.safetensors.index.json.
async fn fetch_safetensors_index(
    client: &reqwest::Client,
    base_url: &str,
    model_id: &str,
) -> Result<Option<u64>, BonaError> {
    let url = format!("{base_url}/{model_id}/resolve/main/model.safetensors.index.json");
    let resp = client.get(&url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND
        || resp.status() == reqwest::StatusCode::UNAUTHORIZED
        || resp.status() == reqwest::StatusCode::FORBIDDEN
    {
        return Ok(None);
    }

    let resp = resp.error_for_status()?;
    let index: SafetensorsIndex = resp
        .json()
        .await
        .map_err(|e| BonaError::Parse(e.to_string()))?;
    Ok(index.metadata.and_then(|m| m.total_size))
}

/// Fetch tokenizer class from tokenizer_config.json.
async fn fetch_tokenizer_config(
    client: &reqwest::Client,
    base_url: &str,
    model_id: &str,
) -> Result<Option<String>, BonaError> {
    let url = format!("{base_url}/{model_id}/resolve/main/tokenizer_config.json");
    let resp = client.get(&url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND
        || resp.status() == reqwest::StatusCode::UNAUTHORIZED
        || resp.status() == reqwest::StatusCode::FORBIDDEN
    {
        return Ok(None);
    }

    let resp = resp.error_for_status()?;
    let config: TokenizerConfig = resp
        .json()
        .await
        .map_err(|e| BonaError::Parse(e.to_string()))?;
    Ok(config.tokenizer_class)
}
