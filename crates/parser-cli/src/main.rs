//! Command-line adapter for deterministic local replay parsing.

use std::{
    error::Error,
    fmt::{self, Display},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use parser_contract::{
    artifact::ParseStatus,
    presence::FieldPresence,
    schema::parse_artifact_schema,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay};
use serde_json::json;
use sha2::{Digest, Sha256};

#[derive(Debug, Parser)]
#[command(name = "replay-parser-2")]
#[command(about = "SolidGames OCAP replay parser")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Parse a local OCAP JSON replay into a parse artifact.
    Parse {
        /// Input OCAP JSON replay path.
        input: PathBuf,
        /// Output parse artifact JSON path.
        #[arg(long)]
        output: PathBuf,
        /// Optional replay identifier to embed in source metadata.
        #[arg(long)]
        replay_id: Option<String>,
    },
    /// Export the current parse artifact JSON Schema.
    Schema {
        /// Output schema JSON path. Writes to stdout when omitted.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Compare selected old and new parser artifacts.
    Compare {
        /// Source replay path selected for comparison.
        #[arg(long)]
        replay: PathBuf,
        /// New parser artifact path.
        #[arg(long)]
        new_artifact: PathBuf,
        /// Old parser artifact path.
        #[arg(long)]
        old_artifact: PathBuf,
        /// Output comparison report path.
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Debug)]
enum CliError {
    ReadInput { path: PathBuf, source: io::Error },
    WriteOutput { path: PathBuf, source: io::Error },
    WriteStdout(io::Error),
    WriteStderr(io::Error),
    Serialize(serde_json::Error),
    ParserInfo(serde_json::Error),
    Checksum(parser_contract::source_ref::ChecksumValueError),
    ComparePlanned,
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadInput { path, source } => {
                write!(formatter, "could not read input: {}: {source}", path.display())
            }
            Self::WriteOutput { path, source } => {
                write!(formatter, "could not write output: {}: {source}", path.display())
            }
            Self::WriteStdout(source) => {
                write!(formatter, "could not write schema to stdout: {source}")
            }
            Self::WriteStderr(source) => {
                write!(formatter, "could not write command summary to stderr: {source}")
            }
            Self::Serialize(source) => {
                write!(formatter, "could not serialize JSON output: {source}")
            }
            Self::ParserInfo(source) => {
                write!(formatter, "could not build parser metadata: {source}")
            }
            Self::Checksum(source) => {
                write!(formatter, "could not build source checksum: {source}")
            }
            Self::ComparePlanned => {
                formatter.write_str("compare command is planned in Phase 5 Plan 02")
            }
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadInput { source, .. }
            | Self::WriteOutput { source, .. }
            | Self::WriteStdout(source)
            | Self::WriteStderr(source) => Some(source),
            Self::Serialize(source) | Self::ParserInfo(source) => Some(source),
            Self::Checksum(source) => Some(source),
            Self::ComparePlanned => None,
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(exit_code) => exit_code,
        Err(error) => report_error(&error),
    }
}

fn run() -> Result<ExitCode, CliError> {
    match Cli::parse().command {
        Commands::Parse { input, output, replay_id } => parse_command(&input, &output, replay_id),
        Commands::Schema { output } => schema_command(output),
        Commands::Compare { replay: _, new_artifact: _, old_artifact: _, output: _ } => {
            Err(CliError::ComparePlanned)
        }
    }
}

fn parse_command(
    input: &Path,
    output: &Path,
    replay_id: Option<String>,
) -> Result<ExitCode, CliError> {
    let bytes = fs::read(input)
        .map_err(|source| CliError::ReadInput { path: input.to_path_buf(), source })?;
    let checksum_hex = sha256_hex(&bytes);
    let source = ReplaySource {
        replay_id,
        source_file: input.display().to_string(),
        checksum: FieldPresence::Present {
            value: SourceChecksum::sha256(checksum_hex).map_err(CliError::Checksum)?,
            source: None,
        },
    };
    let parser = parser_info()?;
    let artifact = parse_replay(ParserInput {
        bytes: &bytes,
        source,
        parser,
        options: ParserOptions::default(),
    });

    let artifact_bytes = serde_json::to_vec_pretty(&artifact).map_err(CliError::Serialize)?;
    write_pretty_json_file(output, artifact_bytes)?;

    if artifact.status == ParseStatus::Failed {
        let summary = artifact.failure.as_ref().map_or_else(
            || "parse failed: no structured failure payload".to_string(),
            |failure| format!("parse failed: {}", failure.message),
        );
        write_stderr_line(&summary).map_err(CliError::WriteStderr)?;
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn schema_command(output: Option<PathBuf>) -> Result<ExitCode, CliError> {
    let schema = parse_artifact_schema();
    if let Some(path) = output {
        let schema_bytes = serde_json::to_vec_pretty(&schema).map_err(CliError::Serialize)?;
        write_pretty_json_file(&path, schema_bytes)?;
    } else {
        let mut stdout = io::stdout().lock();
        serde_json::to_writer_pretty(&mut stdout, &schema).map_err(CliError::Serialize)?;
        stdout.write_all(b"\n").map_err(CliError::WriteStdout)?;
    }

    Ok(ExitCode::SUCCESS)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn parser_info() -> Result<ParserInfo, CliError> {
    serde_json::from_value(json!({
        "name": "replay-parser-2",
        "version": env!("CARGO_PKG_VERSION")
    }))
    .map_err(CliError::ParserInfo)
}

fn write_pretty_json_file(path: &Path, mut output: Vec<u8>) -> Result<(), CliError> {
    output.push(b'\n');
    fs::write(path, output)
        .map_err(|source| CliError::WriteOutput { path: path.to_path_buf(), source })
}

fn report_error(error: &CliError) -> ExitCode {
    match write_stderr_line(&error.to_string()) {
        Ok(()) => ExitCode::FAILURE,
        Err(_) => ExitCode::from(2),
    }
}

fn write_stderr_line(message: &str) -> io::Result<()> {
    let mut stderr = io::stderr().lock();
    stderr.write_all(message.as_bytes())?;
    stderr.write_all(b"\n")
}
