use chrono::Utc;

use crate::{Finding, ModelInvestigation, Severity};

/// Check trust signals: account age, upload history, community activity.
pub fn check(inv: &mut ModelInvestigation) {
    let community = match &inv.community {
        Some(c) => c,
        None => return,
    };

    // New account (< 90 days old).
    if let Some(created) = &community.author_created_at
        && let Ok(created_dt) = chrono::DateTime::parse_from_rfc3339(created)
    {
        let age_days = (Utc::now() - created_dt.to_utc()).num_days();
        if age_days < 90 {
            let author = community.author.as_deref().unwrap_or("unknown");
            inv.findings.push(Finding {
                id: "new_account".into(),
                title: "Uploader account is very new".into(),
                severity: Severity::Medium,
                detail: format!(
                    "Account '{author}' was created {age_days} days ago. \
                     New accounts uploading models warrant extra scrutiny.",
                ),
                evidence_url: Some(format!("https://huggingface.co/{author}")),
            });
        }
    }

    // Low community engagement (no discussions at all).
    if community.discussion_count == Some(0) {
        inv.findings.push(Finding {
            id: "no_community_activity".into(),
            title: "No community discussion".into(),
            severity: Severity::Info,
            detail: "Model has zero discussion threads. \
                     No community review or feedback has occurred."
                .into(),
            evidence_url: Some(format!(
                "https://huggingface.co/{}/discussions",
                inv.model_id
            )),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommunityEvidence, DeclaredFacts, SCHEMA_VERSION};

    #[test]
    fn new_account_produces_medium() {
        let recent = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
        let mut inv = make_inv(Some(&recent), Some(5), Some(3));
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "new_account");
        assert_eq!(inv.findings[0].severity, Severity::Medium);
    }

    #[test]
    fn old_account_produces_no_finding() {
        let old = (Utc::now() - chrono::Duration::days(365)).to_rfc3339();
        let mut inv = make_inv(Some(&old), Some(100), Some(10));
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn zero_discussions_produces_info() {
        let old = (Utc::now() - chrono::Duration::days(365)).to_rfc3339();
        let mut inv = make_inv(Some(&old), Some(100), Some(0));
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "no_community_activity");
        assert_eq!(inv.findings[0].severity, Severity::Info);
    }

    fn make_inv(
        created_at: Option<&str>,
        model_count: Option<u64>,
        discussion_count: Option<u64>,
    ) -> ModelInvestigation {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            model_id: "test/model".into(),
            declared: DeclaredFacts {
                model_id: "test/model".into(),
                ..Default::default()
            },
            lineage: None,
            config: None,
            community: Some(CommunityEvidence {
                author: Some("testuser".into()),
                author_created_at: created_at.map(|s| s.into()),
                author_model_count: model_count,
                discussion_count,
                closed_discussion_count: Some(0),
            }),
            sources: vec![],
            findings: vec![],
        }
    }
}
