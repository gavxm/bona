//! Bona CLI. Parses args, calls `bona::investigate`, and renders the result.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use bona::{ModelInvestigation, SourceStatus};

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
        Command::Investigate { model_id, json } => {
            match bona::investigate(&model_id).await {
                Ok(inv) => {
                    if json {
                        // unwrap is fine here, serializing our own struct won't fail
                        println!("{}", serde_json::to_string_pretty(&inv).unwrap());
                    } else {
                        print_text_report(&inv);
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

/// Render a human-readable text report.
fn print_text_report(inv: &ModelInvestigation) {
    println!("Bona investigation - {}", inv.model_id);
    println!("{}", "=".repeat(50));

    println!("\nDeclared facts:");
    println!(
        "  license:     {}",
        inv.declared
            .declared_license
            .as_deref()
            .unwrap_or("(none declared)")
    );
    println!(
        "  base model:  {}",
        inv.declared
            .declared_base_model
            .as_deref()
            .unwrap_or("(none declared)")
    );
    println!(
        "  library:     {}",
        inv.declared.library.as_deref().unwrap_or("(unknown)")
    );
    println!(
        "  downloads:   {}",
        inv.declared
            .downloads
            .map(|d| d.to_string())
            .unwrap_or_else(|| "(unknown)".into())
    );
    if !inv.declared.tags.is_empty() {
        println!("  tags:        {}", inv.declared.tags.join(", "));
    }

    if let Some(lineage) = &inv.lineage {
        println!("\nLineage:");
        if let Some(parent) = &lineage.parent_id {
            let status = match lineage.parent_exists {
                Some(true) => "",
                Some(false) => " (not found on HF)",
                None => " (not checked)",
            };
            println!("  parent:      {parent}{status}");
            if let Some(license) = &lineage.parent_license {
                println!("  parent license: {license}");
            }
        } else {
            println!("  parent:      (none declared)");
        }
        if !lineage.siblings.is_empty() {
            println!("  siblings:    {}", lineage.siblings.join(", "));
        }
    }

    if let Some(config) = &inv.config {
        println!("\nModel config:");
        if !config.architectures.is_empty() {
            println!("  architecture: {}", config.architectures.join(", "));
        }
        if let Some(model_type) = &config.model_type {
            println!("  model type:   {model_type}");
        }
        if let Some(hidden) = config.hidden_size {
            println!("  hidden size:  {hidden}");
        }
        if let Some(layers) = config.num_hidden_layers {
            println!("  layers:       {layers}");
        }
        if let Some(vocab) = config.vocab_size {
            println!("  vocab size:   {vocab}");
        }
        if let Some(size) = config.safetensors_total_size {
            let gb = size as f64 / 1_000_000_000.0;
            println!("  weight size:  {gb:.1} GB");
        }
        if let Some(tok) = &config.tokenizer_class {
            println!("  tokenizer:    {tok}");
        }
    }

    println!("\nEvidence sources:");
    for rec in &inv.sources {
        let status = match &rec.status {
            SourceStatus::Ok { fetched_ms } => format!("ok ({fetched_ms}ms)"),
            SourceStatus::Failed { reason } => format!("failed: {reason}"),
            SourceStatus::NotImplemented => "not implemented yet".to_string(),
        };
        println!("  {:?}: {}", rec.source, status);
    }

    println!("\nFindings:");
    if inv.findings.is_empty() {
        println!("  (none)");
    } else {
        for f in &inv.findings {
            println!("  [{:?}] {} - {}", f.severity, f.title, f.detail);
            if let Some(url) = &f.evidence_url {
                println!("         evidence: {url}");
            }
        }
    }
}
