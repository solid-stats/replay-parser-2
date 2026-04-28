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
    report.validate().map_err(|error| error.to_string())?;

    Ok(format!(
        "benchmark_report_valid=true\nten_x_status={:?}\nparity_status={:?}\n",
        report.ten_x_status, report.parity_status
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
}

impl Args {
    fn parse(raw_args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut args = Self::default();
        let mut raw_args = raw_args.into_iter();

        while let Some(flag) = raw_args.next() {
            let value = raw_args.next().ok_or_else(|| format!("{flag} requires a value"))?;
            match flag.as_str() {
                "--report" => args.report = PathBuf::from(value),
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        if args.report.as_os_str().is_empty() {
            return Err("--report is required".to_string());
        }

        Ok(args)
    }
}
