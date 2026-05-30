//! SARIF 2.1.0 output for GitHub code scanning integration.

use crate::{Finding, ModelInvestigation, Severity};

/// Map bona severity to SARIF level.
fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
        Severity::Info => "note",
    }
}

/// Build a SARIF result from a finding.
fn sarif_result(model_id: &str, finding: &Finding) -> serde_json::Value {
    let mut result = serde_json::json!({
        "ruleId": finding.id,
        "level": sarif_level(finding.severity),
        "message": {
            "text": format!("[{}] {}: {}", model_id, finding.title, finding.detail)
        },
        "properties": {
            "model_id": model_id,
            "severity": finding.severity,
            "reason": finding.reason,
        }
    });

    if let Some(url) = &finding.evidence_url {
        result["locations"] = serde_json::json!([{
            "physicalLocation": {
                "artifactLocation": {
                    "uri": url
                }
            }
        }]);
    }

    result
}

/// Convert one or more investigations to SARIF JSON.
pub fn to_sarif(investigations: &[&ModelInvestigation]) -> String {
    let mut results = Vec::new();
    let mut rules = std::collections::BTreeMap::new();

    for inv in investigations {
        for finding in &inv.findings {
            results.push(sarif_result(&inv.model_id, finding));

            rules.entry(finding.id.clone()).or_insert_with(|| {
                serde_json::json!({
                    "id": finding.id,
                    "shortDescription": { "text": finding.title },
                    "defaultConfiguration": {
                        "level": sarif_level(finding.severity)
                    }
                })
            });
        }
    }

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "bona",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/gavxm/bona",
                    "rules": rules.into_values().collect::<Vec<_>>()
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap()
}
