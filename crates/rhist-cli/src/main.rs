use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rhist_core::package_content_hash;
use rhist_io::read_package_dir;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rhist")]
#[command(about = "RHIST unit-history package verifier")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Verify(VerifyArgs),
}

#[derive(Debug, Parser)]
struct VerifyArgs {
    package_dir: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "pretty-json")]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    PrettyJson,
}

#[derive(Debug, Serialize)]
struct VerifyTranscript {
    status: VerificationStatus,
    package_id: Option<String>,
    package_content_hash: Option<String>,
    checks: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum VerificationStatus {
    Pass,
    Fail,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    match Cli::parse().command {
        Commands::Verify(args) => run_verify(args),
    }
}

fn run_verify(args: VerifyArgs) -> Result<i32> {
    let (code, transcript) = match read_package_dir(&args.package_dir) {
        Ok(package) => {
            let checks = rhist_core::verify_package(&package)?
                .into_iter()
                .map(|report| report.check_id.to_string())
                .collect();
            (
                0,
                VerifyTranscript {
                    status: VerificationStatus::Pass,
                    package_id: Some(package.manifest.package_id.clone()),
                    package_content_hash: Some(package_content_hash(&package)?),
                    checks,
                    error: None,
                },
            )
        }
        Err(err) => (
            1,
            VerifyTranscript {
                status: VerificationStatus::Fail,
                package_id: None,
                package_content_hash: None,
                checks: Vec::new(),
                error: Some(err.to_string()),
            },
        ),
    };

    let output = match args.format {
        OutputFormat::Json => serde_json::to_string(&transcript)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(&transcript)?,
    };
    if let Some(path) = &args.output {
        std::fs::write(path, output).with_context(|| format!("writing {}", path.display()))?;
    } else {
        println!("{output}");
    }
    Ok(code)
}
