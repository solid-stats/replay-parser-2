//! Benchmark report validation CLI used by `scripts/benchmark-phase5.sh`.

use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use parser_harness::benchmark_report::BenchmarkReport;

fn main() -> ExitCode {
    match run() {
        Ok(summary) => write_stdout(&summary),
        Err(error) => write_stderr(&error),
    }
}

fn run() -> Result<String, String> {
    let args = Args::parse(env::args().skip(1))?;
    let report_json = fs::read_to_string(&args.report)
        .map_err(|error| format!("could not read report {}: {error}", args.report.display()))?;
    let report: BenchmarkReport = serde_json::from_str(&report_json)
        .map_err(|error| format!("benchmark report JSON is invalid: {error}"))?;
    args.mode.validate(&report).map_err(|error| error.to_string())?;

    Ok(format!(
        "benchmark_report_valid=true\n{}benchmark_report_mode={}\nphase={}\nartifact_size_limit_bytes={}\nselected_x3_status={:?}\nselected_parity_status={:?}\nselected_artifact_size_status={:?}\nall_raw_x10_status={:?}\nall_raw_size_gate_status={:?}\nall_raw_zero_failure_status={:?}\n",
        args.mode.acceptance_summary_line(),
        args.mode.as_str(),
        report.phase,
        report.artifact_size_limit_bytes,
        report.selected_large_replay.x3_status,
        report.selected_large_replay.parity_status,
        report.selected_large_replay.artifact_size_status,
        report.all_raw_corpus.x10_status,
        report.all_raw_corpus.size_gate_status,
        report.all_raw_corpus.zero_failure_status
    ))
}

fn write_stdout(message: &str) -> ExitCode {
    let mut stdout = io::stdout().lock();
    match stdout.write_all(message.as_bytes()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(2),
    }
}

fn write_stderr(message: &str) -> ExitCode {
    let mut stderr = io::stderr().lock();
    if stderr.write_all(message.as_bytes()).is_ok() && stderr.write_all(b"\n").is_ok() {
        ExitCode::FAILURE
    } else {
        ExitCode::from(2)
    }
}

#[derive(Debug, Default)]
struct Args {
    report: PathBuf,
    mode: CheckMode,
}

impl Args {
    fn parse(raw_args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut args = Self::default();
        let mut raw_args = raw_args.into_iter();

        while let Some(flag) = raw_args.next() {
            let value = raw_args.next().ok_or_else(|| format!("{flag} requires a value"))?;
            match flag.as_str() {
                "--report" => args.report = PathBuf::from(value),
                "--mode" => args.mode = CheckMode::parse(&value)?,
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        if args.report.as_os_str().is_empty() {
            return Err("--report is required".to_string());
        }

        Ok(args)
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum CheckMode {
    #[default]
    Acceptance,
    Structural,
}

impl CheckMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "acceptance" => Ok(Self::Acceptance),
            "structural" => Ok(Self::Structural),
            other => Err(format!("unknown --mode value: {other}")),
        }
    }

    fn validate(self, report: &BenchmarkReport) -> Result<(), impl ToString> {
        match self {
            Self::Acceptance => report.validate_acceptance(),
            Self::Structural => report.validate(),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Acceptance => "acceptance",
            Self::Structural => "structural",
        }
    }

    const fn acceptance_summary_line(self) -> &'static str {
        match self {
            Self::Acceptance => "benchmark_report_acceptance=true\n",
            Self::Structural => "",
        }
    }
}
