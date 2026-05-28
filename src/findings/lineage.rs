use crate::{Finding, ModelInvestigation, Severity};

/// Known model type → architecture family mappings.
fn architecture_family(model_type: &str) -> Option<&'static str> {
    match model_type.to_lowercase().as_str() {
        "llama" | "codellama" => Some("llama"),
        "mistral" | "mixtral" => Some("mistral"),
        "phi" | "phi3" | "phimoe" => Some("phi"),
        "gemma" | "gemma2" | "gemma3" => Some("gemma"),
        "gpt2" | "gpt_neo" | "gpt_neox" | "gptj" => Some("gpt"),
        "falcon" | "refinedweb" => Some("falcon"),
        "bloom" => Some("bloom"),
        "mpt" => Some("mpt"),
        "qwen" | "qwen2" | "qwen2_5" | "qwen2_5_vl" | "qwen3" | "qwen3_moe" => Some("qwen"),
        "stablelm" => Some("stablelm"),
        "starcoder2" => Some("starcoder"),
        "t5" | "mt5" | "flan-t5" => Some("t5"),
        "bert" | "roberta" | "distilbert" | "electra" => Some("bert"),
        _ => None,
    }
}

/// Extract likely architecture family from a base model id.
/// ex. "meta-llama/Llama-3.1-8B" → look for "llama" in the name.
fn family_from_model_id(model_id: &str) -> Option<&'static str> {
    let lower = model_id.to_lowercase();
    // Check the most distinctive names first.
    if lower.contains("llama") || lower.contains("codellama") {
        return Some("llama");
    }
    if lower.contains("mixtral") || lower.contains("mistral") {
        return Some("mistral");
    }
    if lower.contains("gemma") {
        return Some("gemma");
    }
    if lower.contains("phi-") || lower.contains("phi2") || lower.contains("phi3") {
        return Some("phi");
    }
    if lower.contains("falcon") {
        return Some("falcon");
    }
    if lower.contains("qwen") {
        return Some("qwen");
    }
    if lower.contains("bloom") {
        return Some("bloom");
    }
    if lower.contains("starcoder") {
        return Some("starcoder");
    }
    None
}

/// Check for lineage inconsistency: declared base model vs actual architecture.
pub fn check(inv: &mut ModelInvestigation) {
    let lineage = match &inv.lineage {
        Some(l) => l,
        None => return,
    };

    let parent_id = match lineage.parent_id.as_deref() {
        Some(id) => id,
        None => return,
    };

    let parent_exists = lineage.parent_exists;

    let model_type = match inv.config.as_ref().and_then(|c| c.model_type.as_deref()) {
        Some(mt) => mt,
        None => return, // No config data - can't cross-reference.
    };

    let config_family = architecture_family(model_type);
    let parent_family = family_from_model_id(parent_id);

    match (config_family, parent_family) {
        (Some(cf), Some(pf)) if cf != pf => {
            let exists_note = if parent_exists == Some(false) {
                " (parent model not found on HuggingFace)"
            } else {
                ""
            };
            inv.findings.push(Finding {
                id: "lineage_inconsistency".into(),
                title: "Lineage inconsistency".into(),
                severity: Severity::High,
                detail: format!(
                    "Declares base model '{}' ({} family) but config.json shows \
                     model_type '{}' ({} family).{exists_note}",
                    parent_id, pf, model_type, cf,
                ),
                evidence_url: Some(format!("https://huggingface.co/{parent_id}")),
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn architecture_family_classifies_known_types() {
        assert_eq!(architecture_family("llama"), Some("llama"));
        assert_eq!(architecture_family("mistral"), Some("mistral"));
        assert_eq!(architecture_family("phi"), Some("phi"));
        assert_eq!(architecture_family("gemma2"), Some("gemma"));
        assert_eq!(architecture_family("qwen2"), Some("qwen"));
        assert_eq!(architecture_family("unknown_arch"), None);
    }

    #[test]
    fn family_from_model_id_extracts_family() {
        assert_eq!(
            family_from_model_id("meta-llama/Llama-3.1-8B"),
            Some("llama")
        );
        assert_eq!(
            family_from_model_id("mistralai/Mistral-7B-v0.1"),
            Some("mistral")
        );
        assert_eq!(family_from_model_id("some/random-model"), None);
    }

    #[test]
    fn detects_family_mismatch() {
        let mut inv = make_inv("meta-llama/Llama-3.1-8B", "phi");
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "lineage_inconsistency");
        assert_eq!(inv.findings[0].severity, Severity::High);
    }

    #[test]
    fn matching_family_produces_no_finding() {
        let mut inv = make_inv("meta-llama/Llama-3.1-8B", "llama");
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn unknown_family_produces_no_finding() {
        let mut inv = make_inv("some/unknown-model", "llama");
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    fn make_inv(parent_id: &str, model_type: &str) -> ModelInvestigation {
        use crate::{DeclaredFacts, ModelConfigEvidence, ModelTreeEvidence, SCHEMA_VERSION};

        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            model_id: "test/child".into(),
            declared: DeclaredFacts {
                model_id: "test/child".into(),
                declared_base_model: Some(parent_id.into()),
                ..Default::default()
            },
            lineage: Some(ModelTreeEvidence {
                parent_id: Some(parent_id.into()),
                parent_license: None,
                parent_exists: Some(true),
                siblings: vec![],
            }),
            config: Some(ModelConfigEvidence {
                model_type: Some(model_type.into()),
                ..Default::default()
            }),
            community: None,
            sources: vec![],
            findings: vec![],
        }
    }
}
