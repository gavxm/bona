//! Metadata anomaly checks. Detects mismatches between tags and config,
//! and weight size anomalies.

use crate::{Finding, ModelInvestigation, Severity, WEIGHT_EXTENSIONS};

/// Check for metadata anomalies: mismatches between declared and actual values.
pub fn check(inv: &mut ModelInvestigation) {
    check_files(inv);

    let config = match &inv.config {
        Some(c) => c,
        None => return,
    };

    // Architecture declared in tags doesn't match config.json.
    if !config.architectures.is_empty() {
        let arch = &config.architectures[0];
        let tags = &inv.declared.tags;
        // HF tags often include the model family (ex. "llama", "phi").
        // If the model type from config doesn't appear anywhere in the tags,
        // that's a mild anomaly.
        if let Some(model_type) = &config.model_type {
            let type_lower = model_type.to_lowercase();
            let tag_match = tags.iter().any(|t| t.to_lowercase() == type_lower);
            if !tag_match && !tags.is_empty() {
                inv.findings.push(Finding {
                    id: "model_type_not_in_tags".into(),
                    title: "Model type missing from tags".into(),
                    severity: Severity::Info,
                    detail: format!(
                        "config.json declares architecture '{}' (type '{}') but \
                         '{}' does not appear in the model tags.",
                        arch, model_type, model_type,
                    ),
                    reason: "Minor metadata inconsistency. Tags are user-maintained \
                             and may simply be incomplete."
                        .into(),
                    declared_value: Some(format!("tags: [{}]", tags.join(", "))),
                    actual_value: Some(format!("model_type: {model_type}")),
                    evidence_url: Some(format!(
                        "https://huggingface.co/{}/blob/main/config.json",
                        inv.model_id,
                    )),
                });
            }
        }
    }

    // Undeclared quantization: config has quantization_config but no quant tag.
    if let Some(ref method) = config.quant_method {
        let quant_tags = ["gptq", "awq", "bnb", "bitsandbytes", "exl2", "gguf", "eetq"];
        let tags_lower: Vec<String> = inv.declared.tags.iter().map(|t| t.to_lowercase()).collect();
        let has_quant_tag = tags_lower
            .iter()
            .any(|t| quant_tags.iter().any(|q| t.contains(q)));
        if !has_quant_tag {
            let bits_info = config
                .quant_bits
                .map(|b| format!(" ({b}-bit)"))
                .unwrap_or_default();
            inv.findings.push(Finding {
                id: "undeclared_quantization".into(),
                title: "Quantization not declared in tags".into(),
                severity: Severity::Low,
                detail: format!(
                    "config.json has quantization_config with method '{method}'{bits_info} \
                     but no quantization-related tag is present.",
                ),
                reason: "Undeclared quantization makes it harder for users to know \
                         they are downloading a quantized model rather than the original."
                    .into(),
                declared_value: Some(format!("tags: [{}]", inv.declared.tags.join(", "))),
                actual_value: Some(format!("quant_method: {method}{bits_info}")),
                evidence_url: Some(format!(
                    "https://huggingface.co/{}/blob/main/config.json",
                    inv.model_id,
                )),
            });
        }
    }

    // Parameter count tag vs config estimate.
    // Tags like "7b", "13b", "70b" claim a parameter count that can be
    // cross-referenced against the architecture dimensions in config.json.
    if let (Some(hidden), Some(layers), Some(vocab)) = (
        config.hidden_size,
        config.num_hidden_layers,
        config.vocab_size,
    ) {
        let estimated_params = 2 * vocab * hidden + layers * 12 * hidden * hidden;
        let tag_params = extract_param_count_from_tags(&inv.declared.tags);
        if let Some(claimed) = tag_params {
            // Allow 50% tolerance — the formula is rough.
            let lower = claimed / 2;
            let upper = claimed * 2;
            if estimated_params < lower || estimated_params > upper {
                let claimed_b = claimed as f64 / 1e9;
                let est_b = estimated_params as f64 / 1e9;
                inv.findings.push(Finding {
                    id: "parameter_count_mismatch".into(),
                    title: "Parameter count contradicts tags".into(),
                    severity: Severity::Low,
                    detail: format!(
                        "Tags claim ~{claimed_b:.1}B parameters but config.json dimensions \
                         (hidden={hidden}, layers={layers}, vocab={vocab}) suggest \
                         ~{est_b:.1}B parameters.",
                    ),
                    reason: "A significant mismatch between tagged parameter count and \
                             architecture dimensions may indicate mislabeled tags or a \
                             modified architecture."
                        .into(),
                    declared_value: Some(format!("~{claimed_b:.1}B (from tags)")),
                    actual_value: Some(format!("~{est_b:.1}B (estimated from config)")),
                    evidence_url: Some(format!(
                        "https://huggingface.co/{}/blob/main/config.json",
                        inv.model_id,
                    )),
                });
            }
        }
    }

    // Safetensors total size vs common param count thresholds.
    // Rough heuristic: 2 bytes per param (fp16). If the weight size is
    // wildly different from what the architecture suggests, flag it.
    if let (Some(total_size), Some(hidden), Some(layers), Some(vocab)) = (
        config.safetensors_total_size,
        config.hidden_size,
        config.num_hidden_layers,
        config.vocab_size,
    ) {
        // Very rough param estimate: (vocab*hidden + layers*(12*hidden^2) + hidden*vocab)
        // This is approximate for transformer models.
        let estimated_params = 2 * vocab * hidden + layers * 12 * hidden * hidden;
        let estimated_bytes_fp16 = estimated_params * 2;

        // If actual size is less than half or more than 3x the estimate, flag it.
        if total_size > 0
            && estimated_bytes_fp16 > 0
            && (total_size < estimated_bytes_fp16 / 2 || total_size > estimated_bytes_fp16 * 3)
        {
            let est_b = estimated_bytes_fp16 as f64 / 1e9;
            let actual_b = total_size as f64 / 1e9;
            inv.findings.push(Finding {
                id: "weight_size_anomaly".into(),
                title: "Weight size anomaly".into(),
                severity: Severity::Low,
                detail: format!(
                    "Safetensors total size ({actual_b:.1} GB) differs significantly from \
                     estimated size ({est_b:.1} GB) based on architecture parameters \
                     (hidden={hidden}, layers={layers}, vocab={vocab}).",
                ),
                reason: "Weight size mismatch may indicate quantization, pruning, \
                         or a mislabeled architecture. Not necessarily malicious."
                    .into(),
                declared_value: Some(format!("{est_b:.1} GB (estimated from config)")),
                actual_value: Some(format!("{actual_b:.1} GB (safetensors)")),
                evidence_url: Some(format!(
                    "https://huggingface.co/{}/blob/main/model.safetensors.index.json",
                    inv.model_id,
                )),
            });
        }
    }
}

/// Unsafe/pickle-capable weight extensions. These formats can execute
/// arbitrary code when loaded via `torch.load` or similar.
const UNSAFE_WEIGHT_EXTENSIONS: &[&str] = &[".bin", ".pkl", ".pth", ".pt", ".ckpt"];

/// Extract a parameter count from tags like "7b", "13b", "70b", "1.5b", "0.5b".
fn extract_param_count_from_tags(tags: &[String]) -> Option<u64> {
    for tag in tags {
        let lower = tag.to_lowercase();
        // Match patterns like "7b", "13b", "1.5b", "70b", "0.5b"
        if let Some(num_str) = lower.strip_suffix('b') {
            if let Ok(n) = num_str.parse::<f64>() {
                if (0.1..=1000.0).contains(&n) {
                    return Some((n * 1e9) as u64);
                }
            }
        }
    }
    None
}

/// File extensions that should not appear in a model repo.
/// Excludes .dll since ONNX/CUDA runtime DLLs are common in deployment repos.
const SUSPICIOUS_EXTENSIONS: &[&str] = &[".exe", ".bat", ".cmd", ".msi", ".scr"];

/// Check file listing for anomalies.
fn check_files(inv: &mut ModelInvestigation) {
    if inv.declared.files.is_empty() {
        return;
    }

    let has_weights = inv
        .declared
        .files
        .iter()
        .any(|f| WEIGHT_EXTENSIONS.iter().any(|ext| f.ends_with(ext)));

    if !has_weights {
        inv.findings.push(Finding {
            id: "no_weight_files".into(),
            title: "No model weight files found".into(),
            severity: Severity::Medium,
            detail: format!(
                "Model repo contains {} files but none with recognized weight \
                 extensions (.safetensors, .bin, .gguf, etc.).",
                inv.declared.files.len(),
            ),
            reason: "A model repo without weight files may be a placeholder, \
                     a misconfigured upload, or an attempt to distribute \
                     non-model content."
                .into(),
            declared_value: None,
            actual_value: Some(format!("{} files, no weights", inv.declared.files.len())),
            evidence_url: Some(format!("https://huggingface.co/{}/tree/main", inv.model_id)),
        });
    }

    // Unsafe weight format: pickle-capable files that can execute arbitrary code.
    let unsafe_weights: Vec<&str> = inv
        .declared
        .files
        .iter()
        .filter(|f| UNSAFE_WEIGHT_EXTENSIONS.iter().any(|ext| f.ends_with(ext)))
        .map(|s| s.as_str())
        .collect();

    if !unsafe_weights.is_empty() {
        let has_safetensors = inv.declared.files.iter().any(|f| f.ends_with(".safetensors"));
        let (severity, detail) = if has_safetensors {
            (
                Severity::Low,
                format!(
                    "Model repo contains unsafe weight files ({}) alongside safetensors. \
                     The pickle-based files are redundant and carry code execution risk.",
                    unsafe_weights.join(", "),
                ),
            )
        } else {
            (
                Severity::Medium,
                format!(
                    "Model repo uses pickle-based weight files ({}) with no safetensors \
                     alternative. These formats can execute arbitrary code when loaded.",
                    unsafe_weights.join(", "),
                ),
            )
        };
        inv.findings.push(Finding {
            id: "unsafe_weight_format".into(),
            title: "Unsafe weight format".into(),
            severity,
            detail,
            reason: "Pickle-based formats (.bin, .pkl, .pth, .pt, .ckpt) can execute \
                     arbitrary code via torch.load. Prefer safetensors for safe loading."
                .into(),
            declared_value: None,
            actual_value: Some(unsafe_weights.join(", ")),
            evidence_url: Some(format!("https://huggingface.co/{}/tree/main", inv.model_id)),
        });
    }

    let suspicious: Vec<&str> = inv
        .declared
        .files
        .iter()
        .filter(|f| SUSPICIOUS_EXTENSIONS.iter().any(|ext| f.ends_with(ext)))
        .map(|s| s.as_str())
        .collect();

    if !suspicious.is_empty() {
        inv.findings.push(Finding {
            id: "suspicious_files".into(),
            title: "Suspicious file types in model repo".into(),
            severity: Severity::Low,
            detail: format!(
                "Model repo contains files with suspicious extensions: {}",
                suspicious.join(", "),
            ),
            reason: "Executable files in model repositories are unexpected and \
                     could indicate malicious content distribution."
                .into(),
            declared_value: None,
            actual_value: Some(suspicious.join(", ")),
            evidence_url: Some(format!("https://huggingface.co/{}/tree/main", inv.model_id)),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeclaredFacts, ModelConfigEvidence, SCHEMA_VERSION};

    #[test]
    fn weight_size_anomaly_detected() {
        let mut inv = make_inv(4096, 32, 32000, Some(100_000)); // way too small
        check(&mut inv);
        let finding = inv.findings.iter().find(|f| f.id == "weight_size_anomaly");
        assert!(finding.is_some());
    }

    #[test]
    fn reasonable_weight_size_no_finding() {
        // Rough: 2*32000*4096 + 32*12*4096*4096 = ~6.7B params, ~13.4GB fp16
        let mut inv = make_inv(4096, 32, 32000, Some(14_000_000_000));
        check(&mut inv);
        let finding = inv.findings.iter().find(|f| f.id == "weight_size_anomaly");
        assert!(finding.is_none());
    }

    #[test]
    fn undeclared_quantization_detected() {
        let mut inv = make_inv(4096, 32, 32000, Some(14_000_000_000));
        inv.config.as_mut().unwrap().quant_method = Some("gptq".into());
        inv.config.as_mut().unwrap().quant_bits = Some(4);
        // Tags don't include any quant tag.
        inv.declared.tags = vec!["transformers".into(), "llama".into()];
        check(&mut inv);
        assert!(
            inv.findings
                .iter()
                .any(|f| f.id == "undeclared_quantization")
        );
    }

    #[test]
    fn no_undeclared_quantization_when_tagged() {
        let mut inv = make_inv(4096, 32, 32000, Some(14_000_000_000));
        inv.config.as_mut().unwrap().quant_method = Some("gptq".into());
        inv.declared.tags = vec!["transformers".into(), "gptq".into()];
        check(&mut inv);
        assert!(
            !inv.findings
                .iter()
                .any(|f| f.id == "undeclared_quantization")
        );
    }

    #[test]
    fn no_weight_files_detected() {
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.files = vec!["README.md".into(), "config.json".into()];
        check(&mut inv);
        assert!(inv.findings.iter().any(|f| f.id == "no_weight_files"));
    }

    #[test]
    fn no_finding_when_weights_present() {
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.files = vec!["README.md".into(), "model.safetensors".into()];
        check(&mut inv);
        assert!(!inv.findings.iter().any(|f| f.id == "no_weight_files"));
    }

    #[test]
    fn suspicious_files_detected() {
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.files = vec!["model.safetensors".into(), "payload.exe".into()];
        check(&mut inv);
        let finding = inv.findings.iter().find(|f| f.id == "suspicious_files");
        assert!(finding.is_some());
        assert_eq!(finding.unwrap().severity, Severity::Low);
    }

    #[test]
    fn suspicious_files_all_extensions() {
        for ext in [".exe", ".bat", ".cmd", ".msi", ".scr"] {
            let mut inv = make_inv(4096, 32, 32000, None);
            inv.declared.files = vec!["model.safetensors".into(), format!("file{ext}")];
            check(&mut inv);
            assert!(
                inv.findings.iter().any(|f| f.id == "suspicious_files"),
                "{ext} should be flagged"
            );
        }
    }

    #[test]
    fn dll_is_not_flagged_as_suspicious() {
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.files = vec!["model.safetensors".into(), "onnxruntime.dll".into()];
        check(&mut inv);
        assert!(!inv.findings.iter().any(|f| f.id == "suspicious_files"));
    }

    #[test]
    fn empty_files_list_produces_no_file_findings() {
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.files = vec![];
        check(&mut inv);
        assert!(!inv.findings.iter().any(|f| f.id == "no_weight_files"));
        assert!(!inv.findings.iter().any(|f| f.id == "suspicious_files"));
    }

    #[test]
    fn unsafe_weight_without_safetensors_is_medium() {
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.files = vec!["model.bin".into(), "config.json".into()];
        check(&mut inv);
        let finding = inv.findings.iter().find(|f| f.id == "unsafe_weight_format");
        assert!(finding.is_some());
        assert_eq!(finding.unwrap().severity, Severity::Medium);
    }

    #[test]
    fn unsafe_weight_with_safetensors_is_low() {
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.files = vec![
            "model.safetensors".into(),
            "pytorch_model.bin".into(),
        ];
        check(&mut inv);
        let finding = inv.findings.iter().find(|f| f.id == "unsafe_weight_format");
        assert!(finding.is_some());
        assert_eq!(finding.unwrap().severity, Severity::Low);
    }

    #[test]
    fn safetensors_only_no_unsafe_finding() {
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.files = vec!["model.safetensors".into()];
        check(&mut inv);
        assert!(!inv.findings.iter().any(|f| f.id == "unsafe_weight_format"));
    }

    #[test]
    fn parameter_count_mismatch_detected() {
        // Tags say 7b but config suggests ~6.7B — within tolerance, no finding.
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.tags = vec!["transformers".into(), "llama".into(), "7b".into()];
        check(&mut inv);
        assert!(!inv.findings.iter().any(|f| f.id == "parameter_count_mismatch"));
    }

    #[test]
    fn parameter_count_mismatch_large_discrepancy() {
        // Tags say 70b but config dimensions suggest ~6.7B.
        let mut inv = make_inv(4096, 32, 32000, None);
        inv.declared.tags = vec!["transformers".into(), "llama".into(), "70b".into()];
        check(&mut inv);
        assert!(inv.findings.iter().any(|f| f.id == "parameter_count_mismatch"));
    }

    #[test]
    fn extract_param_count_parses_tags() {
        assert_eq!(
            extract_param_count_from_tags(&["7b".into()]),
            Some(7_000_000_000)
        );
        assert_eq!(
            extract_param_count_from_tags(&["1.5b".into()]),
            Some(1_500_000_000)
        );
        assert_eq!(
            extract_param_count_from_tags(&["transformers".into(), "13B".into()]),
            Some(13_000_000_000)
        );
        assert_eq!(
            extract_param_count_from_tags(&["llama".into()]),
            None
        );
    }

    fn make_inv(
        hidden: u64,
        layers: u64,
        vocab: u64,
        total_size: Option<u64>,
    ) -> ModelInvestigation {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            investigated_at: "2025-01-01T00:00:00Z".into(),
            model_id: "test/model".into(),
            declared: DeclaredFacts {
                model_id: "test/model".into(),
                tags: vec!["transformers".into(), "llama".into()],
                ..Default::default()
            },
            lineage: None,
            config: Some(ModelConfigEvidence {
                architectures: vec!["LlamaForCausalLM".into()],
                model_type: Some("llama".into()),
                hidden_size: Some(hidden),
                num_hidden_layers: Some(layers),
                vocab_size: Some(vocab),
                safetensors_total_size: total_size,
                tokenizer_class: None,
                quant_method: None,
                quant_bits: None,
            }),
            community: None,
            sources: vec![],
            findings: vec![],
        }
    }
}
