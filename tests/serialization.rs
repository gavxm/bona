//! JSON serialization round-trip test for schema v3.

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
            "cardData": {
                "license": "mit",
                "base_model": [{ "model": "testorg/parent", "relation": "finetune" }]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/parent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/parent",
            "gated": "manual",
            "cardData": {
                "license": "apache-2.0",
                "base_model": "testorg/grandparent"
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/grandparent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/grandparent",
            "cardData": { "license": "gpl-3.0" }
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

    // Verify lineage is populated before round-trip.
    assert_eq!(inv.lineage.as_ref().unwrap().chain.len(), 2);

    let json_str = serde_json::to_string(&inv).expect("serialize should succeed");
    let inv2: bona::ModelInvestigation =
        serde_json::from_str(&json_str).expect("deserialize should succeed");

    // Schema and identity.
    assert_eq!(inv.schema_version, inv2.schema_version);
    assert_eq!(inv.model_id, inv2.model_id);

    // Declared facts.
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

    // Lineage chain round-trip.
    let chain1 = &inv.lineage.as_ref().unwrap().chain;
    let chain2 = &inv2.lineage.as_ref().unwrap().chain;
    assert_eq!(chain1.len(), chain2.len());
    for (a, b) in chain1.iter().zip(chain2.iter()) {
        assert_eq!(a.model_id, b.model_id);
        assert_eq!(a.license, b.license);
        assert_eq!(a.relation, b.relation);
        assert_eq!(a.exists, b.exists);
        assert_eq!(a.gated, b.gated);
        assert_eq!(a.depth, b.depth);
    }

    // Config.
    assert_eq!(
        inv.config.as_ref().unwrap().quant_method,
        inv2.config.as_ref().unwrap().quant_method
    );
    assert_eq!(
        inv.config.as_ref().unwrap().quant_bits,
        inv2.config.as_ref().unwrap().quant_bits
    );

    // Findings count.
    assert_eq!(inv.findings.len(), inv2.findings.len());
}
