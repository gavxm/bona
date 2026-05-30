//! Integration tests for multi-hop lineage walking, cycle detection,
//! depth capping, and transitive license findings.

mod common;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn multi_hop_chain() {
    let server = MockServer::start().await;
    let base = server.uri();

    // Child -> parent -> grandparent (chain of 2).
    Mock::given(method("GET"))
        .and(path("/api/models/testorg/child"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/child",
            "author": "testorg",
            "tags": [],
            "downloads": 100,
            "cardData": {
                "license": "mit",
                "base_model": "testorg/parent"
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/parent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/parent",
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

    let inv = yurai::investigate_with_base_url("testorg/child", &base)
        .await
        .expect("investigation should succeed");

    let lineage = inv.lineage.as_ref().expect("lineage should be populated");
    assert_eq!(lineage.chain.len(), 2);

    assert_eq!(lineage.chain[0].model_id, "testorg/parent");
    assert_eq!(lineage.chain[0].license.as_deref(), Some("apache-2.0"));
    assert_eq!(lineage.chain[0].depth, 0);
    assert!(lineage.chain[0].exists);

    assert_eq!(lineage.chain[1].model_id, "testorg/grandparent");
    assert_eq!(lineage.chain[1].license.as_deref(), Some("gpl-3.0"));
    assert_eq!(lineage.chain[1].depth, 1);
    assert!(lineage.chain[1].exists);

    // MIT child with GPL grandparent should trigger transitive violation.
    assert!(
        inv.findings
            .iter()
            .any(|f| f.id == "transitive_license_violation"),
        "expected transitive_license_violation, got: {:?}",
        inv.findings.iter().map(|f| &f.id).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn cycle_detection() {
    let server = MockServer::start().await;
    let base = server.uri();

    // A -> B -> A (cycle).
    Mock::given(method("GET"))
        .and(path("/api/models/testorg/model-a"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/model-a",
            "author": "testorg",
            "tags": [],
            "downloads": 100,
            "cardData": {
                "license": "mit",
                "base_model": "testorg/model-b"
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/model-b"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/model-b",
            "cardData": {
                "license": "mit",
                "base_model": "testorg/model-a"
            }
        })))
        .mount(&server)
        .await;

    common::mount_common_mocks(&server, "testorg/model-a").await;
    common::mount_config_mocks(
        &server,
        "testorg/model-a",
        serde_json::json!({
            "architectures": ["LlamaForCausalLM"],
            "model_type": "llama"
        }),
    )
    .await;

    let inv = yurai::investigate_with_base_url("testorg/model-a", &base)
        .await
        .expect("investigation should succeed (no infinite loop)");

    let lineage = inv.lineage.as_ref().expect("lineage should be populated");
    // Should have model-b at depth 0, then stop (model-a is the subject, cycle detected).
    assert_eq!(lineage.chain.len(), 1);
    assert_eq!(lineage.chain[0].model_id, "testorg/model-b");
}

#[tokio::test]
async fn chain_terminates_at_missing_parent() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/child"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/child",
            "author": "testorg",
            "tags": [],
            "downloads": 100,
            "cardData": {
                "license": "mit",
                "base_model": "testorg/parent"
            }
        })))
        .mount(&server)
        .await;

    // Parent exists but declares a grandparent that 404s.
    Mock::given(method("GET"))
        .and(path("/api/models/testorg/parent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/parent",
            "cardData": {
                "license": "apache-2.0",
                "base_model": "testorg/deleted-model"
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/deleted-model"))
        .respond_with(ResponseTemplate::new(404))
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

    let inv = yurai::investigate_with_base_url("testorg/child", &base)
        .await
        .expect("investigation should succeed");

    let lineage = inv.lineage.as_ref().expect("lineage should be populated");
    assert_eq!(lineage.chain.len(), 2);

    assert!(lineage.chain[0].exists);
    assert!(!lineage.chain[1].exists);
    assert_eq!(lineage.chain[1].model_id, "testorg/deleted-model");
}

#[tokio::test]
async fn chain_stops_at_max_depth() {
    let server = MockServer::start().await;
    let base = server.uri();

    // Build a chain 6 levels deep. MAX_LINEAGE_DEPTH is 4, so only 4 nodes
    // should appear in the chain.
    let depth_names: Vec<String> = (0..6).map(|i| format!("testorg/level-{i}")).collect();

    // Subject model.
    Mock::given(method("GET"))
        .and(path("/api/models/testorg/subject"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/subject",
            "author": "testorg",
            "tags": [],
            "downloads": 100,
            "cardData": {
                "license": "mit",
                "base_model": &depth_names[0]
            }
        })))
        .mount(&server)
        .await;

    // Each level points to the next.
    for i in 0..6 {
        let next = if i < 5 {
            serde_json::json!({ "license": "mit", "base_model": &depth_names[i + 1] })
        } else {
            serde_json::json!({ "license": "mit" })
        };
        Mock::given(method("GET"))
            .and(path(format!("/api/models/{}", depth_names[i])))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": &depth_names[i],
                "cardData": next
            })))
            .mount(&server)
            .await;
    }

    common::mount_common_mocks(&server, "testorg/subject").await;
    common::mount_config_mocks(
        &server,
        "testorg/subject",
        serde_json::json!({ "architectures": ["LlamaForCausalLM"], "model_type": "llama" }),
    )
    .await;

    let inv = yurai::investigate_with_base_url("testorg/subject", &base)
        .await
        .expect("investigation should succeed");

    let lineage = inv.lineage.as_ref().expect("lineage should be populated");
    assert_eq!(
        lineage.chain.len(),
        yurai::MAX_LINEAGE_DEPTH as usize,
        "chain should be capped at MAX_LINEAGE_DEPTH, got {:?}",
        lineage
            .chain
            .iter()
            .map(|n| &n.model_id)
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn relation_propagates_through_chain() {
    let server = MockServer::start().await;
    let base = server.uri();

    // Child declares parent as finetune, parent declares grandparent as quantized.
    Mock::given(method("GET"))
        .and(path("/api/models/testorg/child"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/child",
            "author": "testorg",
            "tags": [],
            "downloads": 100,
            "cardData": {
                "license": "apache-2.0",
                "base_model": [{ "model": "testorg/parent", "relation": "finetune" }]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/parent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/parent",
            "cardData": {
                "license": "apache-2.0",
                "base_model": [{ "model": "testorg/grandparent", "relation": "quantized" }]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/grandparent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/grandparent",
            "cardData": { "license": "apache-2.0" }
        })))
        .mount(&server)
        .await;

    common::mount_common_mocks(&server, "testorg/child").await;
    common::mount_config_mocks(
        &server,
        "testorg/child",
        serde_json::json!({ "architectures": ["LlamaForCausalLM"], "model_type": "llama" }),
    )
    .await;

    let inv = yurai::investigate_with_base_url("testorg/child", &base)
        .await
        .expect("investigation should succeed");

    let lineage = inv.lineage.as_ref().expect("lineage should be populated");
    assert_eq!(lineage.chain.len(), 2);

    assert_eq!(lineage.chain[0].model_id, "testorg/parent");
    assert_eq!(lineage.chain[0].relation, yurai::RelationKind::Finetune);

    assert_eq!(lineage.chain[1].model_id, "testorg/grandparent");
    assert_eq!(lineage.chain[1].relation, yurai::RelationKind::Quantization);
}
