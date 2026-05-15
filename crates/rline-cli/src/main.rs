use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use rline_core::{foundation_manifest, validate_manifest};

#[derive(Debug, Parser)]
#[command(name = "rline")]
#[command(about = "Shared RLINE kernel migration helpers")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Emit the foundation RLINE manifest.
    Manifest,
    /// List candidate kernel crates.
    Packages {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let manifest = foundation_manifest();
    validate_manifest(&manifest)?;

    match cli.command {
        Command::Manifest => {
            println!("{}", serde_json::to_string_pretty(&manifest)?);
        }
        Command::Packages { format } => match format {
            OutputFormat::Text => {
                for crate_spec in &manifest.crates {
                    println!(
                        "{}\t{:?}\t{:?}\t{}",
                        crate_spec.name,
                        crate_spec.kind,
                        crate_spec.migration_status,
                        crate_spec.source_path
                    );
                }
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&manifest.crates)?);
            }
        },
    }

    Ok(())
}
