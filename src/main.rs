//! Bona CLI. Parses args, calls `bona::investigate`, and renders the result.

use std::io::IsTerminal;
use std::process::ExitCode;
use std::time::Instant;

use chrono::{DateTime, Utc};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

use bona::{Finding, ModelInvestigation, Severity};

const LAVENDER: owo_colors::Rgb = owo_colors::Rgb(180, 160, 230);
const MAX_TAGS: usize = 5;

#[derive(Parser)]
#[command(
    name = "bona",
    version,
    about = "Forensics-grade provenance explorer for AI models",
    long_about = "Bona investigates the provenance of a HuggingFace model: \
                  its lineage, license inheritance, and trust signals."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Investigate a HuggingFace model by id (ex. meta-llama/Llama-3.1-8B-Instruct).
    Investigate {
        /// The HuggingFace model id (ex. meta-llama/Llama-3.1-8B-Instruct).
        model_id: String,

        /// Emit the full investigation document as JSON instead of a text report.
        #[arg(long, group = "output_format")]
        json: bool,

        /// Emit findings in SARIF format (for GitHub code scanning).
        #[arg(long, group = "output_format")]
        sarif: bool,

        /// Exit with code 1 if any high-severity findings are detected (for CI).
        #[arg(long)]
        fail_on_high: bool,
    },
    /// Investigate multiple models from a file or stdin.
    Batch {
        /// Path to a file with one model id per line, or "-" for stdin.
        #[arg(long, default_value = "-")]
        from: String,

        /// Emit all results as a JSON array instead of a text report.
        #[arg(long)]
        json: bool,

        /// Write SARIF output to this file (alongside the normal stdout output).
        #[arg(long)]
        sarif: Option<String>,

        /// Exit with code 1 if any model has high-severity findings.
        #[arg(long)]
        fail_on_high: bool,
    },
    /// Generate shell completions for bash, zsh, fish, or powershell.
    Completions {
        /// The shell to generate completions for.
        shell: Shell,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    // Respect NO_COLOR env var and detect TTY.
    if std::env::var_os("NO_COLOR").is_some() {
        owo_colors::set_override(false);
    }

    let cli = Cli::parse();

    match cli.command {
        Command::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "bona", &mut std::io::stdout());
            return ExitCode::SUCCESS;
        }
        Command::Investigate {
            model_id,
            json,
            sarif,
            fail_on_high,
        } => {
            if !model_id.contains('/') {
                eprintln!(
                    "{} model id must be in org/name format (ex. meta-llama/Llama-3.1-8B-Instruct)",
                    "error:".red().bold()
                );
                return ExitCode::FAILURE;
            }

            let spinner =
                ProgressBar::with_draw_target(None, indicatif::ProgressDrawTarget::stderr());
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                    .template("{spinner} {msg}")
                    .unwrap(),
            );
            spinner.set_message(format!("investigating {}...", model_id));
            spinner.enable_steady_tick(std::time::Duration::from_millis(80));

            let start = Instant::now();
            let result = bona::investigate(&model_id).await;
            let elapsed = start.elapsed();

            spinner.finish_and_clear();

            match result {
                Ok(inv) => {
                    if sarif {
                        println!("{}", to_sarif(&[&inv]));
                    } else if json {
                        println!("{}", serde_json::to_string_pretty(&inv).unwrap());
                    } else {
                        print_text_report(&inv, elapsed);
                    }
                    if fail_on_high && inv.findings.iter().any(|f| f.severity == Severity::High) {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(e) => {
                    eprintln!("{} {e}", "error:".red().bold());
                    ExitCode::FAILURE
                }
            }
        }
        Command::Batch {
            from,
            json,
            sarif,
            fail_on_high,
        } => {
            // Hint when reading from stdin interactively.
            if from == "-" && std::io::stdin().is_terminal() {
                eprintln!(
                    "{} reading model ids from stdin (one per line, ctrl-d to finish)",
                    "hint:".dimmed()
                );
            }

            let model_ids = match read_model_ids(&from) {
                Ok(ids) => ids,
                Err(e) => {
                    eprintln!("{} {e}", "error:".red().bold());
                    return ExitCode::FAILURE;
                }
            };

            if model_ids.is_empty() {
                eprintln!("{} no model ids provided", "error:".red().bold());
                return ExitCode::FAILURE;
            }

            let total = model_ids.len();
            eprintln!(
                "{} {} model{}...",
                "investigating".dimmed(),
                total,
                if total == 1 { "" } else { "s" }
            );

            // Investigate concurrently with a cap of 4.
            let mut set = tokio::task::JoinSet::new();
            let mut pending: Vec<String> = model_ids.into_iter().rev().collect();
            type BatchResult = (usize, Result<ModelInvestigation, (String, String)>);
            let mut batch_results: Vec<BatchResult> = Vec::new();
            let concurrency = 4;
            let mut order = 0usize;

            while !pending.is_empty() || !set.is_empty() {
                while set.len() < concurrency && !pending.is_empty() {
                    let model_id = pending.pop().unwrap();
                    let idx = order;
                    order += 1;
                    set.spawn(async move {
                        let result = bona::investigate(&model_id).await;
                        (idx, model_id, result)
                    });
                }

                if let Some(Ok((idx, model_id, result))) = set.join_next().await {
                    match result {
                        Ok(inv) => {
                            eprintln!("  {} {}", "✓".green(), model_id.dimmed());
                            batch_results.push((idx, Ok(inv)));
                        }
                        Err(e) => {
                            eprintln!("  {} {}: {e}", "✗".red(), model_id);
                            batch_results.push((idx, Err((model_id, e.to_string()))));
                        }
                    }
                }
            }

            // Sort by original order.
            batch_results.sort_by_key(|(idx, _)| *idx);

            let mut investigations: Vec<ModelInvestigation> = Vec::new();
            let mut has_high = false;
            let mut errors = 0u32;

            for (_, result) in batch_results {
                match result {
                    Ok(inv) => {
                        if inv.findings.iter().any(|f| f.severity == Severity::High) {
                            has_high = true;
                        }
                        investigations.push(inv);
                    }
                    Err(_) => {
                        errors += 1;
                    }
                }
            }

            // Write SARIF to file if requested.
            let mut sarif_failed = false;
            if let Some(sarif_path) = &sarif {
                let refs: Vec<&ModelInvestigation> = investigations.iter().collect();
                let sarif_json = to_sarif(&refs);
                if let Err(e) = std::fs::write(sarif_path, &sarif_json) {
                    eprintln!(
                        "{} writing SARIF to {sarif_path}: {e}",
                        "error:".red().bold()
                    );
                    sarif_failed = true;
                } else {
                    eprintln!("{} SARIF written to {sarif_path}", "ok:".green());
                }
            }

            // Primary output to stdout.
            if json {
                println!("{}", serde_json::to_string_pretty(&investigations).unwrap());
            } else {
                print_batch_report(&investigations, errors);
            }

            if fail_on_high && has_high {
                ExitCode::from(1)
            } else if sarif_failed || (errors > 0 && investigations.is_empty()) {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
    }
}

fn severity_badge(severity: Severity) -> String {
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

/// OSC 8 hyperlink for terminals that support it.
fn hyperlink(url: &str, text: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
}

/// Format an ISO 8601 timestamp as "Mon YYYY (N years/months ago)".
fn format_date(iso: &str) -> String {
    let Ok(dt) = DateTime::parse_from_rfc3339(iso) else {
        return iso.to_string();
    };
    let now = Utc::now();
    let age = now.signed_duration_since(dt.to_utc());
    let days = age.num_days();

    let month = dt.format("%b %Y");
    let ago = if days >= 365 {
        let years = days / 365;
        format!("{years} year{}", if years == 1 { "" } else { "s" })
    } else if days >= 30 {
        let months = days / 30;
        format!("{months} month{}", if months == 1 { "" } else { "s" })
    } else {
        format!("{days} day{}", if days == 1 { "" } else { "s" })
    };

    format!("{month} ({ago} ago)")
}

/// Render a human-readable text report.
fn print_text_report(inv: &ModelInvestigation, elapsed: std::time::Duration) {
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
                bona::SourceStatus::Ok { fetched_ms } => format!("{name} {fetched_ms}ms"),
                bona::SourceStatus::Failed { .. } => format!("{name} ✗"),
                bona::SourceStatus::NotImplemented => format!("{name} n/a"),
            }
        })
        .collect();
    println!("  {}", source_parts.join(" · ").dimmed());

    let has_failures = inv
        .sources
        .iter()
        .any(|r| matches!(r.status, bona::SourceStatus::Failed { .. }));
    if has_failures && std::env::var_os("HF_TOKEN").is_none() {
        println!(
            "  {}",
            "hint: some sources failed - this model may be gated. set HF_TOKEN for full access."
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

fn print_batch_report(results: &[ModelInvestigation], errors: u32) {
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

        // Show individual findings.
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

/// Wrap text at word boundaries to fit within `width` columns.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() > width {
            lines.push(current);
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Format a number with comma separators.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Read model IDs from a file or stdin ("-").
fn read_model_ids(path: &str) -> Result<Vec<String>, String> {
    use std::io::BufRead;

    let reader: Box<dyn std::io::BufRead> = if path == "-" {
        Box::new(std::io::stdin().lock())
    } else {
        let file = std::fs::File::open(path).map_err(|e| format!("could not open {path}: {e}"))?;
        Box::new(std::io::BufReader::new(file))
    };

    let mut ids = Vec::new();
    for (line_num, line) in reader.lines().enumerate() {
        match line {
            Ok(l) => {
                let trimmed = l.trim().to_string();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    ids.push(trimmed);
                }
            }
            Err(e) => {
                return Err(format!("read error at line {}: {e}", line_num + 1));
            }
        }
    }

    // Validate format.
    for id in &ids {
        if !id.contains('/') {
            return Err(format!(
                "invalid model id '{id}': must be in org/name format"
            ));
        }
    }

    Ok(ids)
}

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
fn to_sarif(investigations: &[&ModelInvestigation]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn read_model_ids_parses_valid_file() {
        let f = write_temp("org/model-a\norg/model-b\n");
        let ids = read_model_ids(f.path().to_str().unwrap()).unwrap();
        assert_eq!(ids, vec!["org/model-a", "org/model-b"]);
    }

    #[test]
    fn read_model_ids_skips_comments_and_blanks() {
        let f = write_temp("# comment\norg/model\n\n  \n# another\norg/other\n");
        let ids = read_model_ids(f.path().to_str().unwrap()).unwrap();
        assert_eq!(ids, vec!["org/model", "org/other"]);
    }

    #[test]
    fn read_model_ids_trims_whitespace() {
        let f = write_temp("  org/model  \n");
        let ids = read_model_ids(f.path().to_str().unwrap()).unwrap();
        assert_eq!(ids, vec!["org/model"]);
    }

    #[test]
    fn read_model_ids_rejects_invalid_format() {
        let f = write_temp("no-slash\n");
        let err = read_model_ids(f.path().to_str().unwrap()).unwrap_err();
        assert!(err.contains("invalid model id"));
    }

    #[test]
    fn read_model_ids_returns_error_for_missing_file() {
        let err = read_model_ids("/nonexistent/path.txt").unwrap_err();
        assert!(err.contains("could not open"));
    }

    #[test]
    fn read_model_ids_returns_empty_for_all_comments() {
        let f = write_temp("# just comments\n# nothing else\n");
        let ids = read_model_ids(f.path().to_str().unwrap()).unwrap();
        assert!(ids.is_empty());
    }
}
