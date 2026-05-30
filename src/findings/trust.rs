//! Trust signal checks. Flags new uploader accounts and zero community
//! engagement.

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
                reason: "New accounts are a common vector for re-uploading models \
                         with stripped licenses or injected weights."
                    .into(),
                declared_value: None,
                actual_value: Some(format!("{age_days} days old")),
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
            reason: "Community engagement is a weak trust signal. \
                     Absence alone is not a risk, but combined with other findings it adds context."
                .into(),
            declared_value: None,
            actual_value: Some("0 discussions".into()),
            evidence_url: Some(format!(
                "https://huggingface.co/{}/discussions",
                inv.model_id
            )),
        });
    }

    // Low engagement: significant downloads but zero likes.
    if let (Some(downloads), Some(likes)) = (inv.declared.downloads, inv.declared.likes) {
        if downloads >= 1000 && likes == 0 {
            inv.findings.push(Finding {
                id: "low_engagement".into(),
                title: "High downloads but zero likes".into(),
                severity: Severity::Info,
                detail: format!(
                    "Model has {downloads} downloads but 0 likes. \
                     Genuine popular models typically accumulate some likes.",
                ),
                reason: "A large download count with zero community endorsement \
                         can indicate automated downloads or scraped re-uploads."
                    .into(),
                declared_value: Some(format!("{downloads} downloads")),
                actual_value: Some("0 likes".into()),
                evidence_url: Some(format!("https://huggingface.co/{}", inv.model_id)),
            });
        }
    }

    // Recently modified old model.
    check_recently_modified(inv);
}

/// Flag old models that were modified very recently. Could indicate model
/// replacement or weight injection.
fn check_recently_modified(inv: &mut ModelInvestigation) {
    let created = match &inv.declared.created_at {
        Some(c) => c,
        None => return,
    };
    let modified = match &inv.declared.last_modified {
        Some(m) => m,
        None => return,
    };

    let Ok(created_dt) = chrono::DateTime::parse_from_rfc3339(created) else {
        return;
    };
    let Ok(modified_dt) = chrono::DateTime::parse_from_rfc3339(modified) else {
        return;
    };

    let age_days = (Utc::now() - created_dt.to_utc()).num_days();
    let since_modified = (Utc::now() - modified_dt.to_utc()).num_days();

    // Old model (> 180 days) modified very recently (< 7 days).
    if age_days > 180 && since_modified < 7 {
        inv.findings.push(Finding {
            id: "recently_modified".into(),
            title: "Old model was recently modified".into(),
            severity: Severity::Info,
            detail: format!(
                "Model was created {age_days} days ago but modified \
                 {since_modified} days ago. Recent changes to old models \
                 may indicate updates, but could also signal replacement.",
            ),
            reason: "Model replacement or weight injection in established repos \
                     is a known supply chain attack vector."
                .into(),
            declared_value: Some(format!("created {age_days} days ago")),
            actual_value: Some(format!("modified {since_modified} days ago")),
            evidence_url: Some(format!(
                "https://huggingface.co/{}/commits/main",
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

    #[test]
    fn low_engagement_high_downloads_zero_likes() {
        let old = (Utc::now() - chrono::Duration::days(365)).to_rfc3339();
        let mut inv = make_inv(Some(&old), Some(100), Some(10));
        inv.declared.downloads = Some(5000);
        inv.declared.likes = Some(0);
        check(&mut inv);
        assert!(inv.findings.iter().any(|f| f.id == "low_engagement"));
    }

    #[test]
    fn no_low_engagement_when_likes_present() {
        let old = (Utc::now() - chrono::Duration::days(365)).to_rfc3339();
        let mut inv = make_inv(Some(&old), Some(100), Some(10));
        inv.declared.downloads = Some(5000);
        inv.declared.likes = Some(10);
        check(&mut inv);
        assert!(!inv.findings.iter().any(|f| f.id == "low_engagement"));
    }

    #[test]
    fn recently_modified_old_model() {
        let old_created = (Utc::now() - chrono::Duration::days(365)).to_rfc3339();
        let recent_modified = (Utc::now() - chrono::Duration::days(2)).to_rfc3339();
        let mut inv = make_inv(Some(&old_created), Some(100), Some(10));
        inv.declared.created_at = Some(old_created);
        inv.declared.last_modified = Some(recent_modified);
        check(&mut inv);
        assert!(inv.findings.iter().any(|f| f.id == "recently_modified"));
    }

    #[test]
    fn no_recently_modified_for_new_model() {
        let recent = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
        let modified = (Utc::now() - chrono::Duration::days(1)).to_rfc3339();
        let mut inv = make_inv(Some(&recent), Some(100), Some(10));
        inv.declared.created_at = Some(recent);
        inv.declared.last_modified = Some(modified);
        check(&mut inv);
        assert!(!inv.findings.iter().any(|f| f.id == "recently_modified"));
    }

    fn make_inv(
        created_at: Option<&str>,
        model_count: Option<u64>,
        discussion_count: Option<u64>,
    ) -> ModelInvestigation {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            investigated_at: "2025-01-01T00:00:00Z".into(),
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
