//! Training data checks. Flags known-restricted datasets and missing
//! dataset declarations.

use crate::{Finding, ModelInvestigation, Severity};

/// Datasets known to carry restrictive terms or legal risk.
/// Each entry: (pattern to match in dataset ID, human label, reason).
const RESTRICTED_DATASETS: &[(&str, &str, &str)] = &[
    (
        "sharegpt",
        "ShareGPT",
        "ShareGPT data contains OpenAI outputs, which are subject to OpenAI's \
         Terms of Use restricting use in competing models.",
    ),
    (
        "openassistant/oasst",
        "OpenAssistant",
        "OpenAssistant (OASST) is released under Apache-2.0 but contains \
         some GPT-generated content that may conflict with OpenAI's terms.",
    ),
    (
        "gpt4all",
        "GPT4All",
        "GPT4All training data includes GPT-3.5/4 outputs, subject to \
         OpenAI's Terms of Use.",
    ),
    (
        "gpt-4",
        "GPT-4 distillation",
        "Datasets containing GPT-4 outputs are subject to OpenAI's Terms \
         of Use, which restrict use in competing models.",
    ),
    (
        "chatgpt",
        "ChatGPT distillation",
        "Datasets containing ChatGPT outputs are subject to OpenAI's Terms \
         of Use, which restrict use in competing models.",
    ),
];

/// Check training dataset declarations.
pub fn check(inv: &mut ModelInvestigation) {
    // Missing datasets: fine-tuned model with no training data declared.
    if inv.declared.datasets.is_empty() && inv.declared.declared_base_model.is_some() {
        inv.findings.push(Finding {
            id: "missing_datasets".into(),
            title: "No training datasets declared".into(),
            severity: Severity::Info,
            detail: "Model declares a base model (suggesting fine-tuning) but lists \
                     no training datasets. Training data is important for assessing \
                     data provenance and license compliance."
                .into(),
            reason: "Without declared training data, downstream users cannot evaluate \
                     whether the fine-tuning data introduces license or content restrictions."
                .into(),
            declared_value: None,
            actual_value: Some("(no datasets field)".into()),
            evidence_url: Some(format!("https://huggingface.co/{}", inv.model_id)),
        });
    }

    if inv.declared.datasets.is_empty() {
        return;
    }

    let mut flagged: Vec<(&str, &str)> = Vec::new();
    for dataset in &inv.declared.datasets {
        let lower = dataset.to_lowercase();
        for &(pattern, label, _reason) in RESTRICTED_DATASETS {
            if lower.contains(pattern) {
                flagged.push((label, pattern));
                break;
            }
        }
    }

    if !flagged.is_empty() {
        let labels: Vec<&str> = flagged.iter().map(|(l, _)| *l).collect();
        // Find the first matching entry to use its reason.
        let reason = RESTRICTED_DATASETS
            .iter()
            .find(|(p, _, _)| flagged.iter().any(|(_, fp)| fp == p))
            .map(|(_, _, r)| *r)
            .unwrap_or("Dataset may carry restrictive terms.");

        inv.findings.push(Finding {
            id: "restricted_training_data".into(),
            title: "Training data may carry restrictions".into(),
            severity: Severity::Medium,
            detail: format!(
                "Declared training datasets include {}, which may carry \
                 usage restrictions that conflict with the model's declared license.",
                labels.join(", "),
            ),
            reason: reason.into(),
            declared_value: Some(format!("datasets: [{}]", inv.declared.datasets.join(", "))),
            actual_value: Some(format!("flagged: {}", labels.join(", "))),
            evidence_url: Some(format!("https://huggingface.co/{}", inv.model_id)),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeclaredFacts, SCHEMA_VERSION};

    #[test]
    fn sharegpt_dataset_flagged() {
        let mut inv = make_inv(vec!["anon8231489123/ShareGPT_Vicuna_unfiltered".into()]);
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "restricted_training_data");
        assert_eq!(inv.findings[0].severity, Severity::Medium);
    }

    #[test]
    fn clean_datasets_no_finding() {
        let mut inv = make_inv(vec!["tatsu-lab/alpaca".into(), "databricks/dolly".into()]);
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn empty_datasets_no_finding_without_base_model() {
        let mut inv = make_inv(vec![]);
        check(&mut inv);
        assert!(inv.findings.is_empty());
    }

    #[test]
    fn missing_datasets_with_base_model() {
        let mut inv = make_inv(vec![]);
        inv.declared.declared_base_model = Some("org/base".into());
        check(&mut inv);
        assert_eq!(inv.findings.len(), 1);
        assert_eq!(inv.findings[0].id, "missing_datasets");
        assert_eq!(inv.findings[0].severity, Severity::Info);
    }

    #[test]
    fn datasets_present_with_base_model_no_missing() {
        let mut inv = make_inv(vec!["tatsu-lab/alpaca".into()]);
        inv.declared.declared_base_model = Some("org/base".into());
        check(&mut inv);
        assert!(inv.findings.iter().all(|f| f.id != "missing_datasets"));
    }

    #[test]
    fn case_insensitive_match() {
        let mut inv = make_inv(vec!["user/SHAREGPT-cleaned".into()]);
        check(&mut inv);
        assert!(
            inv.findings
                .iter()
                .any(|f| f.id == "restricted_training_data")
        );
    }

    fn make_inv(datasets: Vec<String>) -> ModelInvestigation {
        ModelInvestigation {
            schema_version: SCHEMA_VERSION,
            investigated_at: "2025-01-01T00:00:00Z".into(),
            model_id: "test/model".into(),
            declared: DeclaredFacts {
                model_id: "test/model".into(),
                datasets,
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
