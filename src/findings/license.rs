use crate::{Finding, ModelInvestigation, Severity};

/// How restrictive a license is. Higher = more restrictive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Restrictiveness {
    /// MIT, Apache-2.0, BSD, ISC, etc.
    Permissive,
    /// LGPL, MPL, weak copyleft.
    WeakCopyleft,
    /// GPL, AGPL, strong copyleft.
    StrongCopyleft,
    /// Non-commercial: cc-by-nc-*, openrail++, etc.
    NonCommercial,
    /// Custom restricted: llama*, gemma, etc.
    CustomRestricted,
}

/// Classify a license string into a restrictiveness tier.
fn classify(license: &str) -> Option<Restrictiveness> {
    let l = license.to_lowercase();
    match l.as_str() {
        // Permissive.
        "mit" | "apache-2.0" | "bsd-2-clause" | "bsd-3-clause" | "isc" | "unlicense"
        | "cc0-1.0" | "cc-by-4.0" | "wtfpl" | "zlib" | "0bsd" => Some(Restrictiveness::Permissive),

        // Weak copyleft.
        "lgpl-2.1" | "lgpl-3.0" | "mpl-2.0" | "osl-3.0" | "eupl-1.2" | "cc-by-sa-4.0"
        | "cc-by-sa-3.0" => Some(Restrictiveness::WeakCopyleft),

        // Strong copyleft.
        "gpl-2.0" | "gpl-3.0" | "agpl-3.0" => Some(Restrictiveness::StrongCopyleft),

        // Non-commercial.
        "cc-by-nc-4.0"
        | "cc-by-nc-sa-4.0"
        | "cc-by-nc-nd-4.0"
        | "cc-by-nc-3.0"
        | "openrail++"
        | "bigscience-openrail-m"
        | "bigscience-bloom-rail-1.0"
        | "creativeml-openrail-m" => Some(Restrictiveness::NonCommercial),

        // Custom restricted.
        "gemma" => Some(Restrictiveness::CustomRestricted),

        _ => None,
    }
    .or_else(|| {
        // Pattern matches for families.
        if l.starts_with("llama") || l.starts_with("meta-llama") {
            Some(Restrictiveness::CustomRestricted)
        } else if l.starts_with("cc-by-nc") {
            Some(Restrictiveness::NonCommercial)
        } else {
            None
        }
    })
}

/// Human-readable label for a license.
fn license_label(license: &str) -> &str {
    let l = license.to_lowercase();
    if l.starts_with("llama") {
        return "Meta Community License";
    }
    if l == "gemma" {
        return "Google Gemma Terms of Use";
    }
    license
}

/// Check for license inheritance violations: declared license vs parent's license.
pub fn check(inv: &mut ModelInvestigation) {
    let declared_license = match &inv.declared.declared_license {
        Some(l) => l,
        None => return, // No declared license - handled by documentation gap check.
    };

    let parent_license = match inv
        .lineage
        .as_ref()
        .and_then(|l| l.parent_license.as_deref())
    {
        Some(l) => l,
        None => return, // No parent or no parent license - can't cross-reference.
    };

    let parent_id = inv
        .lineage
        .as_ref()
        .and_then(|l| l.parent_id.as_deref())
        .unwrap_or("unknown");

    // Same license - no issue.
    if declared_license.to_lowercase() == parent_license.to_lowercase() {
        return;
    }

    let child_class = classify(declared_license);
    let parent_class = classify(parent_license);

    match (child_class, parent_class) {
        // Child is less restrictive than parent - clear violation.
        (Some(child_r), Some(parent_r)) if child_r < parent_r => {
            inv.findings.push(Finding {
                id: "license_inheritance_violation".into(),
                title: "License inheritance violation".into(),
                severity: Severity::High,
                detail: format!(
                    "Declares '{}' but parent {} uses '{}' ({}). \
                     A derivative cannot grant more permissive rights than the original license allows.",
                    declared_license,
                    parent_id,
                    parent_license,
                    license_label(parent_license),
                ),
                evidence_url: Some(format!(
                    "https://huggingface.co/{parent_id}"
                )),
            });
        }

        // Different licenses, same tier or child more restrictive - warn but lower severity.
        (Some(_), Some(_)) => {
            inv.findings.push(Finding {
                id: "license_mismatch".into(),
                title: "License differs from parent".into(),
                severity: Severity::Low,
                detail: format!(
                    "Declares '{}' while parent {} uses '{}'.",
                    declared_license, parent_id, parent_license,
                ),
                evidence_url: Some(format!("https://huggingface.co/{parent_id}")),
            });
        }

        // One or both unclassified - can't determine, flag as info.
        _ => {
            inv.findings.push(Finding {
                id: "license_unverifiable".into(),
                title: "License inheritance could not be verified".into(),
                severity: Severity::Info,
                detail: format!(
                    "Declares '{}', parent {} uses '{}'. \
                     One or both licenses are not in the known license database.",
                    declared_license, parent_id, parent_license,
                ),
                evidence_url: Some(format!("https://huggingface.co/{parent_id}")),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_known_licenses() {
        assert_eq!(classify("mit"), Some(Restrictiveness::Permissive));
        assert_eq!(classify("MIT"), Some(Restrictiveness::Permissive));
        assert_eq!(classify("apache-2.0"), Some(Restrictiveness::Permissive));
        assert_eq!(classify("gpl-3.0"), Some(Restrictiveness::StrongCopyleft));
        assert_eq!(
            classify("cc-by-nc-4.0"),
            Some(Restrictiveness::NonCommercial)
        );
        assert_eq!(
            classify("llama3.1"),
            Some(Restrictiveness::CustomRestricted)
        );
        assert_eq!(classify("gemma"), Some(Restrictiveness::CustomRestricted));
        assert_eq!(classify("some-unknown-license"), None);
    }

    #[test]
    fn detects_permissive_over_restricted_parent() {
        let mut inv = make_inv("mit", "llama3.1");
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "license_inheritance_violation");
        assert_eq!(inv.findings[0].severity, Severity::High);
    }

    #[test]
    fn same_license_produces_no_finding() {
        let mut inv = make_inv("llama3.1", "llama3.1");
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn different_license_same_tier_produces_low() {
        let mut inv = make_inv("mit", "apache-2.0");
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "license_mismatch");
        assert_eq!(inv.findings[0].severity, Severity::Low);
    }

    #[test]
    fn unknown_license_produces_info() {
        let mut inv = make_inv("mit", "some-custom-thing");
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "license_unverifiable");
        assert_eq!(inv.findings[0].severity, Severity::Info);
    }

    fn make_inv(child_license: &str, parent_license: &str) -> ModelInvestigation {
        use crate::{DeclaredFacts, ModelTreeEvidence, SCHEMA_VERSION};

        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            model_id: "test/child".into(),
            declared: DeclaredFacts {
                model_id: "test/child".into(),
                declared_license: Some(child_license.into()),
                declared_base_model: Some("test/parent".into()),
                ..Default::default()
            },
            lineage: Some(ModelTreeEvidence {
                parent_id: Some("test/parent".into()),
                parent_license: Some(parent_license.into()),
                parent_exists: Some(true),
                siblings: vec![],
            }),
            config: None,
            community: None,
            sources: vec![],
            findings: vec![],
        }
    }
}
