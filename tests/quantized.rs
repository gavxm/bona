//! Integration tests for quantization detection. Verifies that
//! quantization config is extracted from config.json and the
//! undeclared quantization finding fires when tags are missing.

mod common;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn quantization_config_extracted() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/quantized"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/quantized",
            "author": "testorg",
            "tags": ["transformers"],
            "downloads": 500,
            "cardData": { "license": "mit" }
        })))
        .mount(&server)
        .await;

    common::mount_common_mocks(&server, "testorg/quantized").await;
    common::mount_config_mocks(
        &server,
        "testorg/quantized",
        serde_json::json!({
            "architectures": ["LlamaForCausalLM"],
            "model_type": "llama",
            "hidden_size": 4096,
            "vocab_size": 32000,
            "num_hidden_layers": 32,
            "quantization_config": {
                "quant_method": "gptq",
                "bits": 4
            }
        }),
    )
    .await;

    let inv = bona::investigate_with_base_url("testorg/quantized", &base)
        .await
        .expect("investigation should succeed");

    let config = inv.config.expect("config should be populated");
    assert_eq!(config.quant_method.as_deref(), Some("gptq"));
    assert_eq!(config.quant_bits, Some(4));

    assert!(
        inv.findings
            .iter()
            .any(|f| f.id == "undeclared_quantization"),
        "expected undeclared_quantization finding, got: {:?}",
        inv.findings.iter().map(|f| &f.id).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn quantization_with_tag_does_not_fire() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("GET"))
        .and(path("/api/models/testorg/tagged-quant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "testorg/tagged-quant",
            "author": "testorg",
            "tags": ["transformers", "gptq"],
            "downloads": 500,
            "cardData": { "license": "mit" }
        })))
        .mount(&server)
        .await;

    common::mount_common_mocks(&server, "testorg/tagged-quant").await;
    common::mount_config_mocks(
        &server,
        "testorg/tagged-quant",
        serde_json::json!({
            "architectures": ["LlamaForCausalLM"],
            "model_type": "llama",
            "quantization_config": {
                "quant_method": "gptq",
                "bits": 4
            }
        }),
    )
    .await;

    let inv = bona::investigate_with_base_url("testorg/tagged-quant", &base)
        .await
        .expect("investigation should succeed");

    assert!(
        !inv.findings
            .iter()
            .any(|f| f.id == "undeclared_quantization")
    );
}
