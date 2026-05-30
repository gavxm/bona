//! Bona CLI. Thin dispatch layer over the engine and output modules.

use std::io::IsTerminal;
use std::process::ExitCode;
use std::time::Instant;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use owo_colors::OwoColorize;

use bona::output::{sarif, text};
use bona::{ModelInvestigation, Severity};

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
    if std::env::var_os("NO_COLOR").is_some() {
        owo_colors::set_override(false);
    }

    let cli = Cli::parse();

    match cli.command {
        Command::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "bona", &mut std::io::stdout());
            ExitCode::SUCCESS
        }
        Command::Investigate {
            model_id,
            json,
            sarif: sarif_flag,
            fail_on_high,
        } => run_investigate(&model_id, json, sarif_flag, fail_on_high).await,
        Command::Batch {
            from,
            json,
            sarif: sarif_path,
            fail_on_high,
        } => run_batch(&from, json, sarif_path, fail_on_high).await,
    }
}

async fn run_investigate(
    model_id: &str,
    json: bool,
    sarif_flag: bool,
    fail_on_high: bool,
) -> ExitCode {
    if !model_id.contains('/') {
        eprintln!(
            "{} model id must be in org/name format (ex. meta-llama/Llama-3.1-8B-Instruct)",
            "error:".red().bold()
        );
        return ExitCode::FAILURE;
    }

    let spinner =
        indicatif::ProgressBar::with_draw_target(None, indicatif::ProgressDrawTarget::stderr());
    spinner.set_style(
        indicatif::ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("investigating {}...", model_id));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let start = Instant::now();
    let result = bona::investigate(model_id).await;
    let elapsed = start.elapsed();

    spinner.finish_and_clear();

    match result {
        Ok(inv) => {
            if sarif_flag {
                println!("{}", sarif::to_sarif(&[&inv]));
            } else if json {
                println!("{}", serde_json::to_string_pretty(&inv).unwrap());
            } else {
                text::print_text_report(&inv, elapsed);
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

async fn run_batch(
    from: &str,
    json: bool,
    sarif_path: Option<String>,
    fail_on_high: bool,
) -> ExitCode {
    if from == "-" && std::io::stdin().is_terminal() {
        eprintln!(
            "{} reading model ids from stdin (one per line, ctrl-d to finish)",
            "hint:".dimmed()
        );
    }

    let model_ids = match read_model_ids(from) {
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
    if let Some(sarif_file) = &sarif_path {
        let refs: Vec<&ModelInvestigation> = investigations.iter().collect();
        let sarif_json = sarif::to_sarif(&refs);
        if let Err(e) = std::fs::write(sarif_file, &sarif_json) {
            eprintln!(
                "{} writing SARIF to {sarif_file}: {e}",
                "error:".red().bold()
            );
            sarif_failed = true;
        } else {
            eprintln!("{} SARIF written to {sarif_file}", "ok:".green());
        }
    }

    // Primary output to stdout.
    if json {
        println!("{}", serde_json::to_string_pretty(&investigations).unwrap());
    } else {
        text::print_batch_report(&investigations, errors);
    }

    if fail_on_high && has_high {
        ExitCode::from(1)
    } else if sarif_failed || (errors > 0 && investigations.is_empty()) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
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

    for id in &ids {
        if !id.contains('/') {
            return Err(format!(
                "invalid model id '{id}': must be in org/name format"
            ));
        }
    }

    Ok(ids)
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
