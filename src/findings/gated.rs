use crate::{EvidenceSource, Finding, ModelInvestigation, Severity, SourceStatus};

/// Check for gated-derivative patterns: a publicly accessible model that
/// derives from a gated parent. This suggests the uploader may have stripped
/// the gate (access control) from a restricted model.
pub fn check(inv: &mut ModelInvestigation) {
    // We need a declared parent with a restricted license.
    let lineage = match &inv.lineage {
        Some(l) => l,
        None => return,
    };

    let parent_id = match &lineage.parent_id {
        Some(id) => id.as_str(),
        None => return,
    };

    let parent_license = match &lineage.parent_license {
        Some(l) => l.as_str(),
        None => return,
    };

    // Is the parent license one that typically comes with gating?
    let is_restricted = is_typically_gated(parent_license);
    if !is_restricted {
        return;
    }

    // Is the subject model's config publicly accessible? If yes, the derivative
    // is not gated - but the parent likely is.
    let config_succeeded = inv.sources.iter().any(|s| {
        s.source == EvidenceSource::ModelConfig && matches!(s.status, SourceStatus::Ok { .. })
    });

    if config_succeeded {
        inv.findings.push(Finding {
            id: "gated_derivative".into(),
            title: "Derivative of a gated model is publicly accessible".into(),
            severity: Severity::Medium,
            detail: format!(
                "Parent model '{}' uses '{}', which is typically distributed \
                 with access controls (gating). This derivative is publicly \
                 accessible without gating.",
                parent_id, parent_license,
            ),
            reason: "Gated models have access controls for legal or safety reasons. \
                     A public derivative may bypass those restrictions."
                .into(),
            declared_value: Some(format!("{parent_license} (gated)")),
            actual_value: Some("public access (no gate)".into()),
            evidence_url: Some(format!("https://huggingface.co/{parent_id}")),
        });
    }
}

/// Licenses that are typically distributed with HuggingFace gating enabled.
fn is_typically_gated(license: &str) -> bool {
    let l = license.to_lowercase();
    l.starts_with("llama")
        || l == "gemma"
        || l.starts_with("meta-llama")
        || l.contains("community license")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DeclaredFacts, EvidenceSource, ModelConfigEvidence, ModelTreeEvidence, SCHEMA_VERSION,
        SourceRecord, SourceStatus,
    };

    #[test]
    fn detects_public_derivative_of_gated_parent() {
        let mut inv = make_inv("llama3.1", true);
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "gated_derivative");
        assert_eq!(inv.findings[0].severity, Severity::Medium);
    }

    #[test]
    fn no_finding_for_permissive_parent() {
        let mut inv = make_inv("mit", true);
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn no_finding_when_config_failed() {
        let mut inv = make_inv("llama3.1", false);
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    fn make_inv(parent_license: &str, config_ok: bool) -> ModelInvestigation {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            investigated_at: "2025-01-01T00:00:00Z".into(),
            model_id: "test/child".into(),
            declared: DeclaredFacts {
                model_id: "test/child".into(),
                ..Default::default()
            },
            lineage: Some(ModelTreeEvidence {
                parent_id: Some("test/parent".into()),
                parent_license: Some(parent_license.into()),
                parent_exists: Some(true),
                siblings: vec![],
            }),
            config: if config_ok {
                Some(ModelConfigEvidence::default())
            } else {
                None
            },
            community: None,
            sources: vec![SourceRecord {
                source: EvidenceSource::ModelConfig,
                status: if config_ok {
                    SourceStatus::Ok { fetched_ms: 100 }
                } else {
                    SourceStatus::Failed {
                        reason: "401".into(),
                    }
                },
            }],
            findings: vec![],
        }
    }
}
