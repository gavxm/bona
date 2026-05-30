//! Gated-derivative detection. Flags publicly accessible models that derive
//! from parents with gated access controls.

use crate::{EvidenceSource, Finding, ModelInvestigation, Severity, SourceStatus};

/// Check for gated-derivative patterns: a publicly accessible model that
/// derives from a gated parent. This suggests the uploader may have stripped
/// the gate (access control) from a restricted model.
pub fn check(inv: &mut ModelInvestigation) {
    let lineage = match &inv.lineage {
        Some(l) => l,
        None => return,
    };

    let parent_id = match lineage.parent_id() {
        Some(id) => id,
        None => return,
    };

    // Is the child model itself gated? If so, no issue.
    let child_gated = inv
        .declared
        .gated
        .as_deref()
        .is_some_and(|g| g == "auto" || g == "manual" || g == "true");
    if child_gated {
        return;
    }

    // Is the parent gated? Use the direct field, fall back to license heuristic.
    let parent_is_gated = match lineage.parent_gated() {
        Some("auto" | "manual" | "true") => true,
        Some(_) => false,
        None => lineage.parent_license().is_some_and(is_typically_gated),
    };

    if !parent_is_gated {
        return;
    }

    // Is the subject model's config publicly accessible? If yes, the derivative
    // is not gated - but the parent likely is.
    let config_succeeded = inv.sources.iter().any(|s| {
        s.source == EvidenceSource::ModelConfig && matches!(s.status, SourceStatus::Ok { .. })
    });

    if config_succeeded {
        let parent_license_info = lineage
            .parent_license()
            .map(|l| format!(" (license: {l})"))
            .unwrap_or_default();
        inv.findings.push(Finding {
            id: "gated_derivative".into(),
            title: "Derivative of a gated model is publicly accessible".into(),
            severity: Severity::Medium,
            detail: format!(
                "Parent model '{parent_id}' has access controls (gating){parent_license_info}. \
                 This derivative is publicly accessible without gating.",
            ),
            reason: "Gated models have access controls for legal or safety reasons. \
                     A public derivative may bypass those restrictions."
                .into(),
            declared_value: Some("gated parent".into()),
            actual_value: Some("public access (no gate)".into()),
            evidence_url: Some(format!("https://huggingface.co/{parent_id}")),
        });
    }
}

/// Licenses that are typically distributed with HuggingFace gating enabled.
/// Used as a fallback when the direct `gated` field is unavailable.
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
        DeclaredFacts, EvidenceSource, LineageEvidence, LineageNode, ModelConfigEvidence,
        RelationKind, SCHEMA_VERSION, SourceRecord, SourceStatus,
    };

    #[test]
    fn detects_gated_parent_via_direct_field() {
        let mut inv = make_inv(None, Some("manual"), None, true);
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "gated_derivative");
    }

    #[test]
    fn detects_gated_parent_via_license_fallback() {
        let mut inv = make_inv(Some("llama3.1"), None, None, true);
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "gated_derivative");
    }

    #[test]
    fn no_finding_for_permissive_parent() {
        let mut inv = make_inv(Some("mit"), Some("false"), None, true);
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn no_finding_when_config_failed() {
        let mut inv = make_inv(None, Some("manual"), None, false);
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn no_finding_when_child_is_also_gated() {
        let mut inv = make_inv(None, Some("manual"), Some("auto"), true);
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    fn make_inv(
        parent_license: Option<&str>,
        parent_gated: Option<&str>,
        child_gated: Option<&str>,
        config_ok: bool,
    ) -> ModelInvestigation {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            investigated_at: "2025-01-01T00:00:00Z".into(),
            model_id: "test/child".into(),
            declared: DeclaredFacts {
                model_id: "test/child".into(),
                gated: child_gated.map(|s| s.to_string()),
                ..Default::default()
            },
            lineage: Some(LineageEvidence {
                chain: vec![LineageNode {
                    model_id: "test/parent".into(),
                    license: parent_license.map(|s| s.to_string()),
                    relation: RelationKind::Unknown,
                    exists: true,
                    gated: parent_gated.map(|s| s.to_string()),
                    depth: 0,
                    error: None,
                }],
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
