//! Integration tests for gated model detection. Verifies that the direct
//! `gated` field (string and boolean formats) is captured and that the gated
//! derivative finding fires correctly.

mod common;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn ungated_child_of_gated_parent_fires_finding() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/child"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/child",
            "author": "testorg",
            "tags": [],
            "downloads": 100,
            "gated": false,
            "cardData": {
                "license": "mit",
                "base_model": "testorg/gated-parent"
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/gated-parent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/gated-parent",
            "gated": "manual",
            "cardData": { "license": "llama3.1" }
        })))
        .mount(&server)
        .await;

    common::mount_common_mocks(&server, "testorg/child").await;
    common::mount_config_mocks(
        &server,
        "testorg/child",
        serde_json::json!({
            "architectures": ["LlamaForCausalLM"],
            "model_type": "llama"
        }),
    )
    .await;

    let inv = bona::investigate_with_base_url("testorg/child", &base)
        .await
        .expect("investigation should succeed");

    assert_eq!(inv.declared.gated.as_deref(), Some("false"));

    let lineage = inv.lineage.as_ref().expect("lineage should be populated");
    assert_eq!(lineage.parent_gated(), Some("manual"));

    assert!(
        inv.findings.iter().any(|f| f.id == "gated_derivative"),
        "expected gated_derivative finding, got: {:?}",
        inv.findings.iter().map(|f| &f.id).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn boolean_true_gated_is_normalized() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/child"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/child",
            "author": "testorg",
            "tags": [],
            "downloads": 100,
            "gated": false,
            "cardData": {
                "license": "mit",
                "base_model": "testorg/bool-gated"
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/bool-gated"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/bool-gated",
            "gated": true,
            "cardData": { "license": "gemma" }
        })))
        .mount(&server)
        .await;

    common::mount_common_mocks(&server, "testorg/child").await;
    common::mount_config_mocks(
        &server,
        "testorg/child",
        serde_json::json!({
            "architectures": ["GemmaForCausalLM"],
            "model_type": "gemma"
        }),
    )
    .await;

    let inv = bona::investigate_with_base_url("testorg/child", &base)
        .await
        .expect("investigation should succeed");

    let lineage = inv.lineage.as_ref().unwrap();
    assert_eq!(lineage.parent_gated(), Some("true"));
    assert!(inv.findings.iter().any(|f| f.id == "gated_derivative"));
}
