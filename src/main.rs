//! Bona CLI. Parses args, calls `bona::investigate`, and renders the result.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

use bona::{ModelInvestigation, Severity, SourceStatus};

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
        /// The HuggingFace model id.
        model_id: String,

        /// Emit the full investigation document as JSON instead of a text report.
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Investigate { model_id, json } => match bona::investigate(&model_id).await {
            Ok(inv) => {
                if json {
                    println!("{}", serde_json::to_string_pretty(&inv).unwrap());
                } else {
                    print_text_report(&inv);
                }
                let has_high = inv
                    .findings
                    .iter()
                    .any(|f| f.severity == Severity::High);
                if has_high {
                    ExitCode::from(1)
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                eprintln!("{} {e}", "error:".red().bold());
                ExitCode::FAILURE
            }
        },
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
    println!("\n{}", title.bold());
}

fn label(name: &str, value: &str) {
    println!("  {:<15} {}", name.dimmed(), value);
}

/// Render a human-readable text report.
fn print_text_report(inv: &ModelInvestigation) {
    println!(
        "{} {}",
        "Bona investigation".bold(),
        inv.model_id.cyan().bold()
    );
    println!("{}", "─".repeat(60).dimmed());

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
        label("tags", &inv.declared.tags.join(", "));
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
            label("siblings", &lineage.siblings.join(", "));
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
            label("account created", created);
        }
        if let Some(count) = community.author_model_count {
            label("author models", &format_number(count));
        }
        if let Some(count) = community.discussion_count {
            let closed = community.closed_discussion_count.unwrap_or(0);
            label("discussions", &format!("{count} ({closed} closed)"));
        }
    }

    // Evidence sources.
    section_header("Evidence sources");
    for rec in &inv.sources {
        let status = match &rec.status {
            SourceStatus::Ok { fetched_ms } => format!("{} ({}ms)", "ok".green(), fetched_ms),
            SourceStatus::Failed { reason } => format!("{} {reason}", "failed:".red()),
            SourceStatus::NotImplemented => format!("{}", "not implemented".dimmed()),
        };
        println!("  {:<15} {}", format!("{:?}", rec.source).dimmed(), status);
    }

    // Findings.
    section_header("Findings");
    if inv.findings.is_empty() {
        println!("  {}", "No issues found.".green());
    } else {
        for f in &inv.findings {
            println!("\n  {} {}", severity_badge(f.severity), f.title.bold());
            println!("  {}", f.detail);
            if let Some(url) = &f.evidence_url {
                println!("  {} {}", "evidence:".dimmed(), url.underline());
            }
        }
        println!();
        print_summary(inv);
    }
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
        "  {} {total} finding{plural} ({})",
        "Summary:".bold(),
        parts.join(", ")
    );
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
