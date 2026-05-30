use owo_colors::OwoColorize;

use crate::{ModelInvestigation, Severity, SourceStatus};

use super::format::{format_date, format_number, hyperlink, wrap_text};

const LAVENDER: owo_colors::Rgb = owo_colors::Rgb(180, 160, 230);
const MAX_TAGS: usize = 5;

pub fn severity_badge(severity: Severity) -> String {
    match severity {
        Severity::High => format!("{}", " HIGH ".on_red().white().bold()),
        Severity::Medium => format!("{}", " MEDIUM ".on_yellow().black().bold()),
        Severity::Low => format!("{}", " LOW ".on_blue().white().bold()),
        Severity::Info => format!("{}", " INFO ".on_bright_black().white()),
    }
}

fn section_header(title: &str) {
    println!("\n  {}", title.bold());
}

fn label(name: &str, value: &str) {
    println!("  {:<16} {}", name.dimmed(), value);
}

/// Render a human-readable text report for a single investigation.
pub fn print_text_report(inv: &ModelInvestigation, elapsed: std::time::Duration) {
    println!();
    let logo = [
        r"      :+#%%%%%%%%*=. .-*#%%%%%%%#*-.   .:=++++=-:",
        r"   .+%%*-:.    .:=#@%@%+-..   ..-+#%*+%@@@@@@@@@@%*-",
        r"  +@#:           .*@*@%:          .%@@@@@@@@@@@@@@@@%=",
        r" #@=            :%@: .#@-        -@@@@@@:.+@@@@@@@@@@@*",
        r"*@=             %@:    %@.      .@@@@@@@. -*==*@@@@@@@@+",
        r"@@             :@#     =@=      +@@@@@@@. .++: .%@@@@@@@",
        r"@@             :@#     =@=      +@@@@@@@. -@@#  *@@@@@@@",
        r"*@=             %@:    %@.      .@@@@@@@. .==. -@@@@@@@+",
        r" #@=            :%@: .#@-        -@@@@@@##%%**%@@@@@@@*",
        r"  +@#:           .*@*@%:          .%@@@@@@@@@@@@@@@@%=",
        r"   .+%%*-:.    .:=#@%@%+-..   ..-+#%*+%@@@@@@@@@@%*-",
        r"      :+#%%%%%%%%*=. .-*#%%%%%%%#*-.   .:=++++=-:",
    ];
    let width = logo.iter().map(|l| l.len()).max().unwrap_or(0);
    for line in logo {
        println!("{}", line.color(LAVENDER).bold());
    }
    println!();
    let name = [
        r"      __",
        r"     / /  ___  ___  ___ _",
        r"    / _ \/ _ \/ _ \/ _ `/",
        r"   /_.__/\___/_//_/\_,_/",
    ];
    let name_width = name.iter().map(|l| l.len()).max().unwrap_or(0);
    let name_pad = (width - name_width) / 2;
    for line in name {
        println!("{:pad$}{}", "", line.color(LAVENDER).bold(), pad = name_pad);
    }
    let subtitle = "─── provenance explorer ───";
    let visual_len = subtitle.chars().count();
    let pad = (width - visual_len) / 2;
    println!("{:pad$}{}", "", subtitle.dimmed(), pad = pad);

    let hf_url = format!("https://huggingface.co/{}", inv.model_id);
    let model_link = hyperlink(&hf_url, &inv.model_id);
    let ms = elapsed.as_millis();
    println!(
        "\n  {} {}  {}",
        "investigating".dimmed(),
        model_link.cyan().bold(),
        format!("({ms}ms)").dimmed(),
    );
    println!("  {}", "─".repeat(58).dimmed());

    // Declared facts.
    section_header("Declared facts");
    label(
        "license",
        inv.declared
            .declared_license
            .as_deref()
            .unwrap_or("(none declared)"),
    );
    label(
        "base model",
        inv.declared
            .declared_base_model
            .as_deref()
            .unwrap_or("(none declared)"),
    );
    label(
        "library",
        inv.declared.library.as_deref().unwrap_or("(unknown)"),
    );
    label(
        "downloads",
        &inv.declared
            .downloads
            .map(format_number)
            .unwrap_or_else(|| "(unknown)".into()),
    );
    if !inv.declared.tags.is_empty() {
        let tags = &inv.declared.tags;
        if tags.len() <= MAX_TAGS {
            label("tags", &tags.join(", "));
        } else {
            let shown: Vec<&str> = tags.iter().take(MAX_TAGS).map(|s| s.as_str()).collect();
            let remaining = tags.len() - MAX_TAGS;
            label(
                "tags",
                &format!(
                    "{} {}",
                    shown.join(", "),
                    format!("+{remaining} more").dimmed()
                ),
            );
        }
    }

    // Lineage.
    if let Some(lineage) = &inv.lineage {
        section_header("Lineage");
        if let Some(parent) = &lineage.parent_id {
            let status = match lineage.parent_exists {
                Some(true) => "",
                Some(false) => " (not found on HF)",
                None => " (not checked)",
            };
            label("parent", &format!("{parent}{status}"));
            if let Some(license) = &lineage.parent_license {
                label("parent license", license);
            }
        } else {
            label("parent", "(none declared)");
        }
        if !lineage.siblings.is_empty() {
            label(
                "siblings",
                &format!("· {}", lineage.siblings[0]).dimmed().to_string(),
            );
            for sib in &lineage.siblings[1..] {
                println!("  {:<16} {}", "", format!("· {sib}").dimmed());
            }
        }
    }

    // Model config.
    if let Some(config) = &inv.config {
        section_header("Model config");
        if !config.architectures.is_empty() {
            label("architecture", &config.architectures.join(", "));
        }
        if let Some(model_type) = &config.model_type {
            label("model type", model_type);
        }
        if let Some(hidden) = config.hidden_size {
            label("hidden size", &hidden.to_string());
        }
        if let Some(layers) = config.num_hidden_layers {
            label("layers", &layers.to_string());
        }
        if let Some(vocab) = config.vocab_size {
            label("vocab size", &format_number(vocab));
        }
        if let Some(size) = config.safetensors_total_size {
            let gb = size as f64 / 1_000_000_000.0;
            label("weight size", &format!("{gb:.1} GB"));
        }
        if let Some(tok) = &config.tokenizer_class {
            label("tokenizer", tok);
        }
    }

    // Community signals.
    if let Some(community) = &inv.community {
        section_header("Community signals");
        if let Some(author) = &community.author {
            label("author", author);
        }
        if let Some(created) = &community.author_created_at {
            label("account created", &format_date(created));
        }
        if let Some(count) = community.author_model_count {
            label("author models", &format_number(count));
        }
        if let Some(count) = community.discussion_count {
            let closed = community.closed_discussion_count.unwrap_or(0);
            label("discussions", &format!("{count} ({closed} closed)"));
        }
    }

    // Findings.
    println!("\n  {}", "─".repeat(58).dimmed());
    section_header("Findings");
    if inv.findings.is_empty() {
        println!("    {}", "No issues found.".green());
    } else {
        for f in &inv.findings {
            println!();
            println!("    {} {}", severity_badge(f.severity), f.title.bold());
            for line in wrap_text(&f.detail, 54) {
                println!("           {line}");
            }
            if !f.reason.is_empty() {
                for line in wrap_text(&f.reason, 54) {
                    println!("           {}", line.dimmed());
                }
            }
            if let Some(url) = &f.evidence_url {
                println!(
                    "           {} {}",
                    "evidence:".dimmed(),
                    hyperlink(url, url).underline()
                );
            }
        }
        println!();
        print_summary(inv);
    }

    // Evidence sources.
    println!();
    let source_parts: Vec<String> = inv
        .sources
        .iter()
        .map(|rec| {
            let name = format!("{:?}", rec.source);
            match &rec.status {
                SourceStatus::Ok { fetched_ms } => format!("{name} {fetched_ms}ms"),
                SourceStatus::Failed { .. } => format!("{name} ✗"),
                SourceStatus::NotImplemented => format!("{name} n/a"),
            }
        })
        .collect();
    println!("  {}", source_parts.join(" · ").dimmed());

    let has_failures = inv
        .sources
        .iter()
        .any(|r| matches!(r.status, SourceStatus::Failed { .. }));
    if has_failures && std::env::var_os("HF_TOKEN").is_none() {
        println!(
            "  {}",
            "hint: some sources failed — this model may be gated. set HF_TOKEN for full access."
                .yellow()
        );
    }
    println!();
}

fn print_summary(inv: &ModelInvestigation) {
    let mut high = 0u32;
    let mut medium = 0u32;
    let mut low = 0u32;
    let mut info = 0u32;

    for f in &inv.findings {
        match f.severity {
            Severity::High => high += 1,
            Severity::Medium => medium += 1,
            Severity::Low => low += 1,
            Severity::Info => info += 1,
        }
    }

    let total = inv.findings.len();
    let mut parts = Vec::new();
    if high > 0 {
        parts.push(format!("{}", format!("{high} high").red().bold()));
    }
    if medium > 0 {
        parts.push(format!("{}", format!("{medium} medium").yellow().bold()));
    }
    if low > 0 {
        parts.push(format!("{}", format!("{low} low").blue()));
    }
    if info > 0 {
        parts.push(format!("{}", format!("{info} info").dimmed()));
    }

    let plural = if total == 1 { "" } else { "s" };
    println!(
        "    {} {total} finding{plural} ({})",
        "Summary:".bold(),
        parts.join(", ")
    );
}

/// Render a batch results table with individual findings.
pub fn print_batch_report(results: &[ModelInvestigation], errors: u32) {
    println!();
    println!("  {}", "batch results".bold());
    println!("  {}", "─".repeat(58).dimmed());

    for inv in results {
        let high = inv
            .findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .count();
        let med = inv
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Medium)
            .count();
        let total = inv.findings.len();

        let status = if high > 0 {
            format!("{total} findings ({high} high)").red().to_string()
        } else if med > 0 {
            format!("{total} findings ({med} medium)")
                .yellow()
                .to_string()
        } else if total > 0 {
            format!("{total} findings").blue().to_string()
        } else {
            "clean".green().to_string()
        };

        println!("  {:<40} {}", inv.model_id, status);

        for f in &inv.findings {
            let badge = match f.severity {
                Severity::High => "HIGH".red().bold().to_string(),
                Severity::Medium => "MEDIUM".yellow().bold().to_string(),
                Severity::Low => "LOW".blue().to_string(),
                Severity::Info => "INFO".dimmed().to_string(),
            };
            println!("    {} {}", badge, f.title);
        }
    }

    if errors > 0 {
        println!("  {}", format!("{errors} model(s) failed").red());
    }
    println!();
}
