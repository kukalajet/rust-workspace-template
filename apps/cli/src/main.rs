use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use domain::model::Email;

/// Command-line interface for the domain.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a single email address.
    CheckEmail {
        /// The email address to validate.
        #[arg(value_parser = Email::parse)]
        email: Email,
    },
    /// Validate a file containing one email address per line.
    CheckFile {
        /// Path to the file.
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::CheckEmail { email } => {
            println!("valid email: {email}");
            Ok(())
        }
        Command::CheckFile { path } => check_file(&path),
    }
}

fn check_file(path: &Path) -> Result<()> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    for (index, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let line_number = index + 1;
        let email = Email::parse(line).with_context(|| {
            format!("invalid email on line {line_number} of {}", path.display())
        })?;

        println!("line {line_number}: {email}");
    }

    println!("all emails in {} are valid", path.display());

    Ok(())
}
