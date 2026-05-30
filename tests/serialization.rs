//! JSON serialization round-trip test for schema v2.

mod common;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn json_round_trip_preserves_all_fields() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/roundtrip"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/roundtrip",
            "author": "testorg",
            "tags": ["transformers"],
            "downloads": 500,
            "likes": 10,
            "gated": "auto",
            "sha": "deadbeef",
            "lastModified": "2025-05-01T00:00:00.000Z",
            "createdAt": "2024-01-01T00:00:00.000Z",
            "siblings": [{ "rfilename": "model.safetensors" }],
            "cardData": { "license": "mit" }
        })))
        .mount(&server)
        .await;

    common::mount_common_mocks(&server, "testorg/roundtrip").await;
    common::mount_config_mocks(
        &server,
        "testorg/roundtrip",
        serde_json::json!({
            "architectures": ["LlamaForCausalLM"],
            "model_type": "llama",
            "quantization_config": { "quant_method": "awq", "bits": 8 }
        }),
    )
    .await;

    let inv = bona::investigate_with_base_url("testorg/roundtrip", &base)
        .await
        .expect("investigation should succeed");

    let json_str = serde_json::to_string(&inv).expect("serialize should succeed");
    let inv2: bona::ModelInvestigation =
        serde_json::from_str(&json_str).expect("deserialize should succeed");

    assert_eq!(inv.schema_version, inv2.schema_version);
    assert_eq!(inv.model_id, inv2.model_id);
    assert_eq!(inv.declared.likes, inv2.declared.likes);
    assert_eq!(inv.declared.gated, inv2.declared.gated);
    assert_eq!(inv.declared.sha, inv2.declared.sha);
    assert_eq!(inv.declared.last_modified, inv2.declared.last_modified);
    assert_eq!(inv.declared.created_at, inv2.declared.created_at);
    assert_eq!(inv.declared.files, inv2.declared.files);
    assert_eq!(
        inv.declared.base_model_relation,
        inv2.declared.base_model_relation
    );
    assert_eq!(
        inv.config.as_ref().unwrap().quant_method,
        inv2.config.as_ref().unwrap().quant_method
    );
    assert_eq!(
        inv.config.as_ref().unwrap().quant_bits,
        inv2.config.as_ref().unwrap().quant_bits
    );
    assert_eq!(inv.findings.len(), inv2.findings.len());
}
