use crate::{Finding, ModelInvestigation, Severity};

/// Check for metadata anomalies: mismatches between declared and actual values.
pub fn check(inv: &mut ModelInvestigation) {
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
                evidence_url: Some(format!(
                    "https://huggingface.co/{}/blob/main/model.safetensors.index.json",
                    inv.model_id,
                )),
            });
        }
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

    fn make_inv(
        hidden: u64,
        layers: u64,
        vocab: u64,
        total_size: Option<u64>,
    ) -> ModelInvestigation {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
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
            }),
            community: None,
            sources: vec![],
            findings: vec![],
        }
    }
}
