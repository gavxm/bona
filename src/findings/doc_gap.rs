//! Documentation gap checks. Flags missing license or base model declarations.

use crate::{Finding, ModelInvestigation, Severity};

/// Check for documentation gaps: missing card fields that should be present.
pub fn check(inv: &mut ModelInvestigation) {
    let d = &inv.declared;

    if d.declared_license.is_none() {
        inv.findings.push(Finding {
            id: "missing_license".into(),
            title: "No license declared".into(),
            severity: Severity::Medium,
            detail: "Model has no license in its card metadata. \
                     Users cannot determine usage rights."
                .into(),
            reason: "Absence of a license is legally ambiguous. \
                     Default copyright applies, which restricts all use."
                .into(),
            declared_value: None,
            actual_value: Some("(no license field)".into()),
            evidence_url: Some(format!("https://huggingface.co/{}", d.model_id)),
        });
    }

    if d.declared_base_model.is_none() && inv.config.is_some() {
        // Quantized models without a base model are a stronger gap - they are
        // always derivatives, so missing lineage is more concerning.
        let is_quantized = inv
            .config
            .as_ref()
            .and_then(|c| c.quant_method.as_ref())
            .is_some();
        let (severity, detail) = if is_quantized {
            (
                Severity::Medium,
                "Model has quantization config but does not declare a base model. \
                 Quantized models are always derivatives — missing lineage prevents \
                 license and provenance verification.",
            )
        } else {
            (
                Severity::Low,
                "Model has weights but does not declare a base model. \
                 Lineage cannot be verified.",
            )
        };
        inv.findings.push(Finding {
            id: "missing_base_model".into(),
            title: "No base model declared".into(),
            severity,
            detail: detail.into(),
            reason: "Without a declared parent, license inheritance cannot be checked.".into(),
            declared_value: None,
            actual_value: Some("(no base_model field)".into()),
            evidence_url: Some(format!("https://huggingface.co/{}", d.model_id)),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeclaredFacts, ModelConfigEvidence, SCHEMA_VERSION};

    #[test]
    fn missing_license_produces_medium() {
        let mut inv = make_inv(None, Some("org/base"));
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "missing_license");
        assert_eq!(inv.findings[0].severity, Severity::Medium);
    }

    #[test]
    fn missing_base_model_with_config_produces_low() {
        let mut inv = make_inv(Some("mit"), None);
        inv.config = Some(ModelConfigEvidence::default());
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "missing_base_model");
    }

    #[test]
    fn missing_base_model_quantized_produces_medium() {
        let mut inv = make_inv(Some("mit"), None);
        inv.config = Some(ModelConfigEvidence {
            quant_method: Some("gptq".into()),
            ..Default::default()
        });
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "missing_base_model");
        assert_eq!(inv.findings[0].severity, Severity::Medium);
    }

    #[test]
    fn missing_base_model_without_config_produces_nothing() {
        let mut inv = make_inv(Some("mit"), None);
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn complete_metadata_produces_no_finding() {
        let mut inv = make_inv(Some("mit"), Some("org/base"));
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    fn make_inv(license: Option<&str>, base_model: Option<&str>) -> ModelInvestigation {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            investigated_at: "2025-01-01T00:00:00Z".into(),
            model_id: "test/model".into(),
            declared: DeclaredFacts {
                model_id: "test/model".into(),
                declared_license: license.map(|s| s.into()),
                declared_base_model: base_model.map(|s| s.into()),
                ..Default::default()
            },
            lineage: None,
            config: None,
            community: None,
            sources: vec![],
            findings: vec![],
        }
    }
}
