//! Shared wiremock helpers for integration tests.
//!
//! Not every test file uses every helper, so allow dead code here.
#![allow(dead_code)]

use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Mount the standard set of mocks that most tests need: user overview,
/// discussions, and sibling search. Individual tests add model-specific mocks.
pub async fn mount_common_mocks(server: &MockServer, model_id: &str) {
    let org = model_id.split('/').next().unwrap();
    let discussions_path = format!("/api/models/{model_id}/discussions");

    Mock::given(method("GET"))
        .and(path(format!("/api/users/{org}/overview")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "createdAt": "2023-01-15T00:00:00.000Z",
            "numModels": 42
        })))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path(discussions_path))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "discussions": [],
            "count": 7,
            "numClosedDiscussions": 3
        })))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models"))
        .and(query_param("sort", "downloads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(server)
        .await;
}

/// Mount config.json, safetensors index, and tokenizer mocks for a model.
pub async fn mount_config_mocks(
    server: &MockServer,
    model_id: &str,
    config_json: serde_json::Value,
) {
    Mock::given(method("GET"))
        .and(path(format!("/{model_id}/resolve/main/config.json")))
        .respond_with(ResponseTemplate::new(200).set_body_json(config_json))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/{model_id}/resolve/main/model.safetensors.index.json"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "metadata": { "total_size": 14000000000_u64 },
            "weight_map": {}
        })))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/{model_id}/resolve/main/tokenizer_config.json"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tokenizer_class": "LlamaTokenizerFast"
        })))
        .mount(server)
        .await;
}
