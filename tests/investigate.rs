use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn investigate_populates_all_sources() {
    let server = MockServer::start().await;
    let base = server.uri();

    // HF metadata endpoint.
    Mock::given(method("GET"))
        .and(path("/api/models/testorg/testmodel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/testmodel",
            "author": "testorg",
            "license": "mit",
            "library_name": "transformers",
            "pipeline_tag": "text-generation",
            "tags": ["transformers", "license:mit"],
            "downloads": 1000,
            "likes": 15,
            "gated": false,
            "sha": "abc123",
            "lastModified": "2025-03-01T12:00:00.000Z",
            "createdAt": "2024-06-01T00:00:00.000Z",
            "siblings": [
                { "rfilename": "config.json" },
                { "rfilename": "model.safetensors" },
                { "rfilename": "tokenizer.json" }
            ],
            "cardData": {
                "license": "mit",
                "base_model": "testorg/base"
            }
        })))
        .mount(&server)
        .await;

    // Parent model metadata (for model_tree).
    Mock::given(method("GET"))
        .and(path("/api/models/testorg/base"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/base",
            "cardData": {
                "license": "apache-2.0"
            }
        })))
        .mount(&server)
        .await;

    // Sibling search.
    Mock::given(method("GET"))
        .and(path("/api/models"))
        .and(query_param("sort", "downloads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": "testorg/sibling1" },
            { "id": "testorg/testmodel" },
            { "id": "testorg/sibling2" }
        ])))
        .mount(&server)
        .await;

    // config.json
    Mock::given(method("GET"))
        .and(path("/testorg/testmodel/resolve/main/config.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "architectures": ["LlamaForCausalLM"],
            "model_type": "llama",
            "hidden_size": 4096,
            "num_hidden_layers": 32,
            "vocab_size": 32000
        })))
        .mount(&server)
        .await;

    // safetensors index
    Mock::given(method("GET"))
        .and(path(
            "/testorg/testmodel/resolve/main/model.safetensors.index.json",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "metadata": { "total_size": 14000000000_u64 },
            "weight_map": {}
        })))
        .mount(&server)
        .await;

    // tokenizer_config.json
    Mock::given(method("GET"))
        .and(path(
            "/testorg/testmodel/resolve/main/tokenizer_config.json",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tokenizer_class": "LlamaTokenizerFast"
        })))
        .mount(&server)
        .await;

    // User overview.
    Mock::given(method("GET"))
        .and(path("/api/users/testorg/overview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "createdAt": "2023-01-15T00:00:00.000Z",
            "numModels": 42
        })))
        .mount(&server)
        .await;

    // Discussions.
    Mock::given(method("GET"))
        .and(path("/api/models/testorg/testmodel/discussions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "discussions": [],
            "count": 7,
            "numClosedDiscussions": 3
        })))
        .mount(&server)
        .await;

    let inv = bona::investigate_with_base_url("testorg/testmodel", &base)
        .await
        .expect("investigation should succeed");

    // Schema version.
    assert_eq!(inv.schema_version, 2);

    // Declared facts from HF metadata.
    assert_eq!(inv.declared.declared_license.as_deref(), Some("mit"));
    assert_eq!(
        inv.declared.declared_base_model.as_deref(),
        Some("testorg/base")
    );
    assert_eq!(inv.declared.downloads, Some(1000));
    assert_eq!(inv.declared.likes, Some(15));
    assert_eq!(inv.declared.gated.as_deref(), Some("false"));
    assert_eq!(inv.declared.sha.as_deref(), Some("abc123"));
    assert!(
        inv.declared
            .files
            .contains(&"model.safetensors".to_string())
    );
    assert_eq!(inv.declared.files.len(), 3);

    // Model tree.
    let lineage = inv.lineage.expect("lineage should be populated");
    assert_eq!(lineage.parent_id.as_deref(), Some("testorg/base"));
    assert_eq!(lineage.parent_license.as_deref(), Some("apache-2.0"));
    assert_eq!(lineage.parent_exists, Some(true));
    assert!(lineage.siblings.contains(&"testorg/sibling1".to_string()));
    assert!(!lineage.siblings.contains(&"testorg/testmodel".to_string()));

    // Model config.
    let config = inv.config.expect("config should be populated");
    assert_eq!(config.architectures, vec!["LlamaForCausalLM"]);
    assert_eq!(config.model_type.as_deref(), Some("llama"));
    assert_eq!(config.hidden_size, Some(4096));
    assert_eq!(config.safetensors_total_size, Some(14000000000));
    assert_eq!(
        config.tokenizer_class.as_deref(),
        Some("LlamaTokenizerFast")
    );

    // Community signals.
    let community = inv.community.expect("community should be populated");
    assert_eq!(community.author.as_deref(), Some("testorg"));
    assert_eq!(
        community.author_created_at.as_deref(),
        Some("2023-01-15T00:00:00.000Z")
    );
    assert_eq!(community.author_model_count, Some(42));
    assert_eq!(community.discussion_count, Some(7));
    assert_eq!(community.closed_discussion_count, Some(3));

    // All 4 sources should be present and successful.
    assert_eq!(inv.sources.len(), 4);
    for source in &inv.sources {
        assert!(
            matches!(source.status, bona::SourceStatus::Ok { .. }),
            "source {:?} should be Ok, got {:?}",
            source.source,
            source.status
        );
    }

    // Findings: mit child with apache-2.0 parent should produce a license_mismatch.
    let license_finding = inv.findings.iter().find(|f| f.id == "license_mismatch");
    assert!(
        license_finding.is_some(),
        "expected license_mismatch finding, got: {:?}",
        inv.findings.iter().map(|f| &f.id).collect::<Vec<_>>()
    );
}
