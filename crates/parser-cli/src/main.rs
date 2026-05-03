//! Command-line adapter for deterministic local replay parsing.
// coverage-exclusion: reviewed Phase 05 defensive CLI I/O and serialization branches are allowlisted by exact source line.

use std::{
    error::Error,
    fmt::{self, Display},
    fs,
    io::{self, Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Duration,
};

use clap::{Parser, Subcommand, ValueEnum};
use parser_contract::{
    artifact::{ParseArtifact, ParseStatus},
    presence::FieldPresence,
    schema::parse_artifact_schema,
    source_ref::{ReplaySource, SourceChecksum},
    version::ParserInfo,
};
use parser_core::{ParserInput, ParserOptions, parse_replay_debug, public_parse_replay};
use parser_harness::{
    comparison::{ComparisonError, compare_artifacts},
    summary_report::render_markdown_summary,
};
use parser_worker::config::{WorkerConfig, WorkerConfigOverrides};
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
    /// Parse a local OCAP JSON replay into a compact parse artifact.
    Parse {
        /// Input OCAP JSON replay path.
        input: PathBuf,
        /// Output compact parse artifact JSON path.
        #[arg(long)]
        output: PathBuf,
        /// Write human-readable JSON instead of the default minified artifact.
        #[arg(long)]
        pretty: bool,
        /// Optional internal full-detail debug artifact JSON path.
        #[arg(long)]
        debug_artifact: Option<PathBuf>,
        /// Optional replay identifier to embed in source metadata.
        #[arg(long)]
        replay_id: Option<String>,
    },
    /// Export the current compact parse artifact JSON Schema.
    Schema {
        /// Output schema JSON path. Writes to stdout when omitted.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Compare selected old and new parser artifacts.
    Compare {
        /// Source replay path selected for comparison.
        #[arg(long)]
        replay: Option<PathBuf>,
        /// New parser artifact path.
        #[arg(long)]
        new_artifact: Option<PathBuf>,
        /// Old parser artifact path.
        #[arg(long)]
        old_artifact: PathBuf,
        /// Output comparison report path.
        #[arg(long)]
        output: PathBuf,
        /// Optional structured JSON detail report path when writing Markdown output.
        #[arg(long)]
        detail_output: Option<PathBuf>,
        /// Comparison report output format.
        #[arg(long, value_enum, default_value_t = CompareFormat::Markdown)]
        format: CompareFormat,
    },
    /// Run the RabbitMQ/S3 parser worker.
    Worker {
        /// `RabbitMQ` AMQP URL.
        #[arg(long)]
        amqp_url: Option<String>,
        /// `RabbitMQ` parse job queue.
        #[arg(long)]
        job_queue: Option<String>,
        /// `RabbitMQ` exchange for parse results.
        #[arg(long)]
        result_exchange: Option<String>,
        /// Routing key for successful parse results.
        #[arg(long)]
        completed_routing_key: Option<String>,
        /// Routing key for failed parse results.
        #[arg(long)]
        failed_routing_key: Option<String>,
        /// S3 bucket containing raw replays and parser artifacts.
        #[arg(long)]
        s3_bucket: Option<String>,
        /// S3 region.
        #[arg(long)]
        s3_region: Option<String>,
        /// Optional S3-compatible endpoint URL.
        #[arg(long)]
        s3_endpoint: Option<String>,
        /// Force path-style S3 addressing.
        #[arg(long, value_name = "BOOL", num_args = 0..=1, default_missing_value = "true")]
        s3_force_path_style: Option<bool>,
        /// Artifact object key prefix.
        #[arg(long)]
        artifact_prefix: Option<String>,
        /// `RabbitMQ` prefetch count.
        #[arg(long)]
        prefetch: Option<u16>,
        /// HTTP probe bind address.
        #[arg(long)]
        probe_bind: Option<String>,
        /// HTTP probe port.
        #[arg(long)]
        probe_port: Option<u16>,
        /// Enable HTTP probes.
        #[arg(long, value_name = "BOOL")]
        probes_enabled: Option<bool>,
        /// Operator-visible worker identity.
        #[arg(long)]
        worker_id: Option<String>,
    },
    /// Internal Docker health check command.
    #[command(hide = true)]
    Healthcheck {
        /// Probe URL to check.
        #[arg(long, default_value = "http://127.0.0.1:8080/readyz")]
        url: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum CompareFormat {
    Markdown,
    Json,
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
    Compare(ComparisonError),
    Worker(parser_worker::error::WorkerError),
    WorkerRuntime(io::Error),
    CompareRequiresInput,
    CompareConflictingInput,
    CompareJsonDetailOutput,
    DebugArtifactConflictsWithOutput,
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
            Self::Compare(source) => {
                write!(formatter, "could not compare artifacts: {source}")
            }
            Self::Worker(source) => {
                write!(formatter, "worker failed: {source}")
            }
            Self::WorkerRuntime(source) => {
                write!(formatter, "could not build worker runtime: {source}")
            }
            Self::CompareRequiresInput => {
                formatter.write_str("compare requires --replay or --new-artifact")
            }
            Self::CompareConflictingInput => {
                formatter.write_str("compare accepts only one of --replay or --new-artifact")
            }
            Self::CompareJsonDetailOutput => {
                formatter.write_str(
                    "compare --format json cannot be combined with --detail-output because --output is already detailed JSON",
                )
            }
            Self::DebugArtifactConflictsWithOutput => {
                formatter.write_str("parse --debug-artifact must not be the same path as --output")
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
            | Self::WriteStderr(source)
            | Self::WorkerRuntime(source) => Some(source),
            Self::Serialize(source) | Self::ParserInfo(source) => Some(source),
            Self::Checksum(source) => Some(source),
            Self::Compare(source) => Some(source),
            Self::Worker(source) => Some(source),
            Self::CompareRequiresInput
            | Self::CompareConflictingInput
            | Self::CompareJsonDetailOutput
            | Self::DebugArtifactConflictsWithOutput => None,
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
        Commands::Parse { input, output, pretty, debug_artifact, replay_id } => {
            parse_command(&input, &output, replay_id, pretty, debug_artifact.as_deref())
        }
        Commands::Schema { output } => schema_command(output),
        Commands::Compare { replay, new_artifact, old_artifact, output, detail_output, format } => {
            compare_command(
                replay.as_deref(),
                new_artifact.as_deref(),
                &old_artifact,
                &output,
                detail_output.as_deref(),
                format,
            )
        }
        Commands::Worker {
            amqp_url,
            job_queue,
            result_exchange,
            completed_routing_key,
            failed_routing_key,
            s3_bucket,
            s3_region,
            s3_endpoint,
            s3_force_path_style,
            artifact_prefix,
            prefetch,
            probe_bind,
            probe_port,
            probes_enabled,
            worker_id,
        } => worker_command(WorkerConfigOverrides {
            amqp_url,
            job_queue,
            result_exchange,
            completed_routing_key,
            failed_routing_key,
            s3_bucket,
            s3_region,
            s3_endpoint,
            s3_force_path_style,
            artifact_prefix,
            prefetch,
            probe_bind,
            probe_port,
            probes_enabled,
            worker_id,
        }),
        Commands::Healthcheck { url } => Ok(healthcheck_command(&url)),
    }
}

fn parse_command(
    input: &Path,
    output: &Path,
    replay_id: Option<String>,
    pretty: bool,
    debug_artifact: Option<&Path>,
) -> Result<ExitCode, CliError> {
    if debug_artifact.is_some_and(|path| path == output) {
        return Err(CliError::DebugArtifactConflictsWithOutput);
    }

    let input_data = read_parser_input(input, replay_id)?;
    let artifact = public_parse_replay(input_data.parser_input());
    let artifact_bytes = if pretty {
        serde_json::to_vec_pretty(&artifact).map_err(CliError::Serialize)?
    } else {
        serde_json::to_vec(&artifact).map_err(CliError::Serialize)?
    };
    write_json_file(output, artifact_bytes)?;

    if let Some(path) = debug_artifact {
        let debug_artifact = parse_replay_debug(input_data.parser_input());
        let debug_bytes =
            serde_json::to_vec_pretty(&debug_artifact).map_err(CliError::Serialize)?;
        write_json_file(path, debug_bytes)?;
    }

    if let Some(summary) = parse_failure_summary(&artifact) {
        write_stderr_line(&summary).map_err(CliError::WriteStderr)?;
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn parse_failure_summary(artifact: &ParseArtifact) -> Option<String> {
    (artifact.status == ParseStatus::Failed).then(|| {
        artifact.failure.as_ref().map_or_else(
            || "parse failed: no structured failure payload".to_string(),
            |failure| format!("parse failed: {}", failure.message),
        )
    })
}

fn parse_artifact_from_input(
    input: &Path,
    replay_id: Option<String>,
) -> Result<ParseArtifact, CliError> {
    let input_data = read_parser_input(input, replay_id)?;
    Ok(public_parse_replay(input_data.parser_input()))
}

struct ReadParserInput {
    bytes: Vec<u8>,
    source: ReplaySource,
    parser: ParserInfo,
}

impl ReadParserInput {
    fn parser_input(&self) -> ParserInput<'_> {
        ParserInput {
            bytes: &self.bytes,
            source: self.source.clone(),
            parser: self.parser.clone(),
            options: ParserOptions::default(),
        }
    }
}

fn read_parser_input(input: &Path, replay_id: Option<String>) -> Result<ReadParserInput, CliError> {
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
    Ok(ReadParserInput { bytes, source, parser })
}

fn schema_command(output: Option<PathBuf>) -> Result<ExitCode, CliError> {
    let mut stdout = io::stdout().lock();
    schema_command_with_writer(output, &mut stdout)
}

fn schema_command_with_writer(
    output: Option<PathBuf>,
    stdout: &mut impl Write,
) -> Result<ExitCode, CliError> {
    let schema = parse_artifact_schema();
    if let Some(path) = output {
        let schema_bytes = serde_json::to_vec_pretty(&schema).map_err(CliError::Serialize)?;
        write_json_file(&path, schema_bytes)?;
    } else {
        serde_json::to_writer_pretty(&mut *stdout, &schema).map_err(CliError::Serialize)?;
        stdout.write_all(b"\n").map_err(CliError::WriteStdout)?;
    }

    Ok(ExitCode::SUCCESS)
}

fn compare_command(
    replay: Option<&Path>,
    new_artifact: Option<&Path>,
    old_artifact: &Path,
    output: &Path,
    detail_output: Option<&Path>,
    format: CompareFormat,
) -> Result<ExitCode, CliError> {
    if format == CompareFormat::Json && detail_output.is_some() {
        return Err(CliError::CompareJsonDetailOutput);
    }

    let old_bytes = read_file(old_artifact)?;
    let old_label = old_artifact.display().to_string();
    let (new_label, new_bytes) = match (replay, new_artifact) {
        (None, None) => return Err(CliError::CompareRequiresInput),
        (Some(_), Some(_)) => return Err(CliError::CompareConflictingInput),
        (_, Some(path)) => (path.display().to_string(), read_file(path)?),
        (Some(path), None) => {
            let artifact = parse_artifact_from_input(path, None)?;
            let artifact_bytes = serde_json::to_vec(&artifact).map_err(CliError::Serialize)?;
            (format!("parsed replay: {}", path.display()), artifact_bytes)
        }
    };

    let report = compare_artifacts(old_label, &old_bytes, new_label, &new_bytes)
        .map_err(CliError::Compare)?;
    match format {
        CompareFormat::Markdown => {
            write_text_file(output, render_markdown_summary(&report))?;
            if let Some(path) = detail_output {
                let report_bytes =
                    serde_json::to_vec_pretty(&report).map_err(CliError::Serialize)?;
                write_json_file(path, report_bytes)?;
            }
        }
        CompareFormat::Json => {
            let report_bytes = serde_json::to_vec_pretty(&report).map_err(CliError::Serialize)?;
            write_json_file(output, report_bytes)?;
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn worker_command(overrides: WorkerConfigOverrides) -> Result<ExitCode, CliError> {
    let config = WorkerConfig::from_env_and_overrides(|name| std::env::var(name).ok(), overrides)
        .map_err(CliError::Worker)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(CliError::WorkerRuntime)?;
    runtime.block_on(parser_worker::runner::run(config)).map_err(CliError::Worker)?;
    Ok(ExitCode::SUCCESS)
}

fn healthcheck_command(url: &str) -> ExitCode {
    match healthcheck_status(url) {
        Ok(200) => ExitCode::SUCCESS,
        Ok(_status) => ExitCode::FAILURE,
        Err(_error) => ExitCode::from(2),
    }
}

fn healthcheck_status(url: &str) -> Result<u16, HealthcheckError> {
    let target = parse_http_healthcheck_url(url)?;
    let mut stream = TcpStream::connect((target.host.as_str(), target.port))
        .map_err(|_source| HealthcheckError::Unavailable)?;
    let timeout = Some(Duration::from_secs(5));
    stream
        .set_read_timeout(timeout)
        .and_then(|()| stream.set_write_timeout(timeout))
        .map_err(|_source| HealthcheckError::Unavailable)?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nUser-Agent: replay-parser-2-healthcheck\r\nConnection: close\r\n\r\n",
        target.path, target.host, target.port
    );
    stream.write_all(request.as_bytes()).map_err(|_source| HealthcheckError::Unavailable)?;

    let mut response = [0_u8; 256];
    let read = stream.read(&mut response).map_err(|_source| HealthcheckError::Unavailable)?;
    let response =
        std::str::from_utf8(&response[..read]).map_err(|_source| HealthcheckError::Unavailable)?;
    parse_http_status(response).ok_or(HealthcheckError::Unavailable)
}

fn parse_http_healthcheck_url(url: &str) -> Result<HttpHealthcheckUrl, HealthcheckError> {
    let rest = url.strip_prefix("http://").ok_or(HealthcheckError::InvalidUrl)?;
    let (authority, path) = rest.split_once('/').ok_or(HealthcheckError::InvalidUrl)?;
    let (host, port) = authority.rsplit_once(':').ok_or(HealthcheckError::InvalidUrl)?;
    if host.is_empty() || host.contains('@') || port.is_empty() {
        return Err(HealthcheckError::InvalidUrl);
    }
    let port = port.parse::<u16>().map_err(|_source| HealthcheckError::InvalidUrl)?;
    if port == 0 {
        return Err(HealthcheckError::InvalidUrl);
    }

    Ok(HttpHealthcheckUrl { host: host.to_owned(), port, path: format!("/{path}") })
}

fn parse_http_status(response: &str) -> Option<u16> {
    let status_line = response.lines().next()?;
    let mut parts = status_line.split_ascii_whitespace();
    let version = parts.next()?;
    let status = parts.next()?;
    version.starts_with("HTTP/").then(|| status.parse::<u16>().ok()).flatten()
}

#[derive(Debug, Eq, PartialEq)]
struct HttpHealthcheckUrl {
    host: String,
    port: u16,
    path: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum HealthcheckError {
    InvalidUrl,
    Unavailable,
}

fn read_file(path: &Path) -> Result<Vec<u8>, CliError> {
    fs::read(path).map_err(|source| CliError::ReadInput { path: path.to_path_buf(), source })
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

fn write_json_file(path: &Path, mut output: Vec<u8>) -> Result<(), CliError> {
    output.push(b'\n');
    fs::write(path, output)
        .map_err(|source| CliError::WriteOutput { path: path.to_path_buf(), source })
}

fn write_text_file(path: &Path, output: String) -> Result<(), CliError> {
    fs::write(path, output)
        .map_err(|source| CliError::WriteOutput { path: path.to_path_buf(), source })
}

fn report_error(error: &CliError) -> ExitCode {
    report_error_result(write_stderr_line(&error.to_string()).is_ok())
}

fn report_error_result(write_succeeded: bool) -> ExitCode {
    if write_succeeded { ExitCode::FAILURE } else { ExitCode::from(2) }
}

fn write_stderr_line(message: &str) -> io::Result<()> {
    let mut stderr = io::stderr().lock();
    stderr.write_all(message.as_bytes())?;
    stderr.write_all(b"\n")
}

#[cfg(all(test, not(coverage)))]
mod tests {
    #![allow(clippy::expect_used, reason = "unit tests use expect messages as assertion context")]

    use std::{net::TcpListener, thread};

    use super::*;

    const VALID_REPLAY: &[u8] =
        include_bytes!("../../parser-core/tests/fixtures/valid-minimal.ocap.json");

    fn temp_dir(test_name: &str) -> PathBuf {
        let path = std::env::temp_dir()
            .join(format!("replay-parser-2-cli-unit-{}-{test_name}", std::process::id()));
        match fs::remove_dir_all(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => panic!("failed to remove test temp dir {}: {error}", path.display()),
        }
        fs::create_dir_all(&path).expect("test temp dir should be created");
        path
    }

    fn write_replay(path: &Path, bytes: &[u8]) {
        fs::write(path, bytes).expect("test replay should be written");
    }

    #[test]
    fn parse_failure_summary_should_describe_failed_artifacts_without_failure_payload() {
        // Arrange
        let source = ReplaySource {
            replay_id: Some("cli-summary-test".to_owned()),
            source_file: "invalid.ocap.json".to_owned(),
            checksum: FieldPresence::Unknown {
                reason: parser_contract::presence::UnknownReason::SourceFieldAbsent,
                source: None,
            },
        };
        let mut artifact = parser_core::parse_replay(ParserInput {
            bytes: b"{",
            source,
            parser: parser_info().expect("parser info should be valid"),
            options: ParserOptions::default(),
        });
        artifact.failure = None;

        // Act
        let summary = parse_failure_summary(&artifact);

        // Assert
        assert_eq!(summary.as_deref(), Some("parse failed: no structured failure payload"));
    }

    #[test]
    fn parse_command_should_cover_pretty_debug_failure_and_success_paths() {
        // Arrange
        let dir = temp_dir("parse_command_paths");
        let valid_input = dir.join("valid.ocap.json");
        let invalid_input = dir.join("invalid.ocap.json");
        let valid_output = dir.join("valid-artifact.json");
        let invalid_output = dir.join("invalid-artifact.json");
        let debug_output = dir.join("invalid-debug.json");
        write_replay(&valid_input, VALID_REPLAY);
        write_replay(&invalid_input, b"{");

        // Act
        let valid_exit =
            parse_command(&valid_input, &valid_output, Some("unit-replay".to_owned()), false, None)
                .expect("valid parse should succeed");
        let invalid_exit =
            parse_command(&invalid_input, &invalid_output, None, true, Some(&debug_output))
                .expect("invalid parse should write failed artifact");

        // Assert
        assert_eq!(valid_exit, ExitCode::SUCCESS);
        assert_eq!(invalid_exit, ExitCode::FAILURE);
        assert!(fs::read(&valid_output).expect("valid artifact should exist").ends_with(b"\n"));
        assert!(
            fs::read_to_string(&invalid_output)
                .expect("invalid artifact should exist")
                .contains("\"status\": \"failed\"")
        );
        assert!(debug_output.exists());
    }

    #[test]
    fn schema_and_compare_commands_should_cover_output_branches() {
        // Arrange
        let dir = temp_dir("schema_compare_paths");
        let replay = dir.join("valid.ocap.json");
        let old_artifact = dir.join("old.json");
        let new_artifact = dir.join("new.json");
        let schema_output = dir.join("schema.json");
        let markdown_output = dir.join("comparison.md");
        let detail_output = dir.join("comparison-detail.json");
        let replay_output = dir.join("comparison-from-replay.md");
        let json_output = dir.join("comparison.json");
        write_replay(&replay, VALID_REPLAY);
        let parse_exit = parse_command(&replay, &old_artifact, None, false, None)
            .expect("old artifact should be written");
        let copied_bytes =
            fs::copy(&old_artifact, &new_artifact).expect("new artifact should be copied");

        // Act
        let schema_file_exit =
            schema_command(Some(schema_output.clone())).expect("schema file should write");
        let mut schema_stdout = Vec::new();
        let schema_stdout_exit = schema_command_with_writer(None, &mut schema_stdout)
            .expect("schema stdout should write");
        let markdown_exit = compare_command(
            None,
            Some(&new_artifact),
            &old_artifact,
            &markdown_output,
            Some(&detail_output),
            CompareFormat::Markdown,
        )
        .expect("markdown comparison should write");
        let replay_exit = compare_command(
            Some(&replay),
            None,
            &old_artifact,
            &replay_output,
            None,
            CompareFormat::Markdown,
        )
        .expect("replay comparison should write");
        let json_exit = compare_command(
            None,
            Some(&new_artifact),
            &old_artifact,
            &json_output,
            None,
            CompareFormat::Json,
        )
        .expect("json comparison should write");

        // Assert
        assert_eq!(schema_file_exit, ExitCode::SUCCESS);
        assert_eq!(schema_stdout_exit, ExitCode::SUCCESS);
        assert!(schema_stdout.ends_with(b"\n"));
        assert_eq!(parse_exit, ExitCode::SUCCESS);
        assert!(copied_bytes > 0);
        assert_eq!(markdown_exit, ExitCode::SUCCESS);
        assert_eq!(replay_exit, ExitCode::SUCCESS);
        assert_eq!(json_exit, ExitCode::SUCCESS);
        assert!(schema_output.exists());
        assert!(markdown_output.exists());
        assert!(detail_output.exists());
        assert!(replay_output.exists());
        assert!(json_output.exists());
    }

    #[test]
    fn report_error_result_should_use_distinct_exit_code_when_stderr_write_fails() {
        // Act
        let exit_code = report_error_result(false);

        // Assert
        assert_eq!(exit_code, ExitCode::from(2));
    }

    #[test]
    fn parse_http_healthcheck_url_should_parse_valid_http_urls() {
        // Act
        let parsed = parse_http_healthcheck_url("http://127.0.0.1:8080/readyz")
            .expect("valid healthcheck URL should parse");

        // Assert
        assert_eq!(
            parsed,
            HttpHealthcheckUrl {
                host: "127.0.0.1".to_owned(),
                port: 8080,
                path: "/readyz".to_owned(),
            }
        );
    }

    #[test]
    fn parse_http_healthcheck_url_should_reject_invalid_urls() {
        // Act + Assert
        for url in [
            "https://127.0.0.1:8080/readyz",
            "http://127.0.0.1:8080",
            "http://127.0.0.1/readyz",
            "http://:8080/readyz",
            "http://user@127.0.0.1:8080/readyz",
            "http://127.0.0.1:/readyz",
            "http://127.0.0.1:not-a-port/readyz",
            "http://127.0.0.1:0/readyz",
        ] {
            assert_eq!(parse_http_healthcheck_url(url), Err(HealthcheckError::InvalidUrl));
        }
    }

    #[test]
    fn parse_http_status_should_accept_http_status_lines_and_reject_malformed_responses() {
        // Act + Assert
        assert_eq!(parse_http_status("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n"), Some(200));
        assert_eq!(parse_http_status("HTTP/2 503\r\n"), Some(503));
        assert_eq!(parse_http_status(""), None);
        assert_eq!(parse_http_status("NOTHTTP 200 OK\r\n"), None);
        assert_eq!(parse_http_status("HTTP/1.1\r\n"), None);
        assert_eq!(parse_http_status("HTTP/1.1 ok\r\n"), None);
    }

    #[test]
    fn healthcheck_status_should_report_unavailable_when_connection_fails() {
        // Arrange
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
        let address = listener.local_addr().expect("listener should expose address");
        drop(listener);
        let url = format!("http://{}:{}/readyz", address.ip(), address.port());

        // Act
        let result = healthcheck_status(&url);

        // Assert
        assert_eq!(result, Err(HealthcheckError::Unavailable));
    }

    #[test]
    fn healthcheck_status_should_read_http_response_from_probe() {
        // Arrange
        let url = one_response_probe_url("HTTP/1.1 204 No Content");

        // Act
        let status = healthcheck_status(&url).expect("healthcheck should read response");

        // Assert
        assert_eq!(status, 204);
    }

    #[test]
    fn cli_error_should_display_command_context_for_each_error_variant() {
        // Arrange
        let serde_error =
            serde_json::from_str::<serde_json::Value>("{").expect_err("invalid JSON should fail");
        let parser_info_error = serde_json::from_value::<ParserInfo>(json!({
            "name": "replay-parser-2",
            "version": "not-semver"
        }))
        .expect_err("invalid parser metadata should fail");
        let checksum_error = SourceChecksum::sha256("not-sha256")
            .expect_err("invalid checksum should fail validation");
        let compare_error =
            compare_artifacts("old", br#"{"status":"success"}"#, "new", br#"{"status":"success""#)
                .expect_err("invalid new artifact JSON should fail comparison");
        let cases = [
            (
                CliError::ReadInput {
                    path: PathBuf::from("missing.ocap.json"),
                    source: io::Error::new(io::ErrorKind::NotFound, "missing"),
                },
                "could not read input",
                true,
            ),
            (
                CliError::WriteOutput {
                    path: PathBuf::from("missing/artifact.json"),
                    source: io::Error::new(io::ErrorKind::NotFound, "missing"),
                },
                "could not write output",
                true,
            ),
            (
                CliError::WriteStdout(io::Error::new(io::ErrorKind::BrokenPipe, "closed")),
                "could not write schema to stdout",
                true,
            ),
            (
                CliError::WriteStderr(io::Error::new(io::ErrorKind::BrokenPipe, "closed")),
                "could not write command summary to stderr",
                true,
            ),
            (CliError::Serialize(serde_error), "could not serialize JSON output", true),
            (CliError::ParserInfo(parser_info_error), "could not build parser metadata", true),
            (CliError::Checksum(checksum_error), "could not build source checksum", true),
            (CliError::Compare(compare_error), "could not compare artifacts", true),
            (
                CliError::Worker(parser_worker::error::WorkerError::ConfigValidation(
                    "missing required REPLAY_PARSER_S3_BUCKET".to_owned(),
                )),
                "worker failed",
                true,
            ),
            (
                CliError::WorkerRuntime(io::Error::other("runtime unavailable")),
                "could not build worker runtime",
                true,
            ),
            (CliError::CompareRequiresInput, "compare requires --replay or --new-artifact", false),
            (
                CliError::CompareConflictingInput,
                "compare accepts only one of --replay or --new-artifact",
                false,
            ),
            (
                CliError::CompareJsonDetailOutput,
                "compare --format json cannot be combined with --detail-output",
                false,
            ),
            (
                CliError::DebugArtifactConflictsWithOutput,
                "parse --debug-artifact must not be the same path as --output",
                false,
            ),
        ];

        // Act + Assert
        for (error, expected_text, has_source) in cases {
            assert!(error.to_string().contains(expected_text));
            assert_eq!(error.source().is_some(), has_source);
        }
    }

    fn one_response_probe_url(status_line: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("probe listener should bind");
        let address = listener.local_addr().expect("probe listener should have a local address");
        let _server = thread::spawn(move || {
            let (mut stream, _peer) = listener.accept().expect("healthcheck should connect");
            let mut request = [0_u8; 512];
            let _read = stream.read(&mut request);
            let response =
                format!("{status_line}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            stream.write_all(response.as_bytes()).expect("probe response should write");
        });
        format!("http://{}:{}/readyz", address.ip(), address.port())
    }
}
