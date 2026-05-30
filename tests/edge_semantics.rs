//! Integration tests for edge semantics (base model relation types).

mod common;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn finetune_relation_captured() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/finetuned"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/finetuned",
            "author": "testorg",
            "tags": ["transformers"],
            "downloads": 100,
            "cardData": {
                "license": "apache-2.0",
                "base_model": [{ "model": "testorg/base", "relation": "finetune" }]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/base"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/base",
            "cardData": { "license": "apache-2.0" }
        })))
        .mount(&server)
        .await;

    common::mount_common_mocks(&server, "testorg/finetuned").await;
    common::mount_config_mocks(
        &server,
        "testorg/finetuned",
        serde_json::json!({
            "architectures": ["LlamaForCausalLM"],
            "model_type": "llama"
        }),
    )
    .await;

    let inv = bona::investigate_with_base_url("testorg/finetuned", &base)
        .await
        .expect("investigation should succeed");

    assert_eq!(
        inv.declared.base_model_relation,
        Some(bona::RelationKind::Finetune)
    );
    assert_eq!(
        inv.declared.declared_base_model.as_deref(),
        Some("testorg/base")
    );
}
