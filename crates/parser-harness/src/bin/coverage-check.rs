//! Coverage JSON post-processor used by `scripts/coverage-gate.sh`.

use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use parser_harness::coverage::{CoverageAllowlist, evaluate_coverage_json};

fn main() -> ExitCode {
    match run() {
        Ok(report_text) => write_stdout(&report_text),
        Err(error) => write_stderr(&error),
    }
}

fn run() -> Result<String, String> {
    let args = Args::parse(env::args().skip(1))?;
    let allowlist_text = fs::read_to_string(&args.allowlist).map_err(|error| {
        format!("could not read allowlist {}: {error}", args.allowlist.display())
    })?;
    let coverage_json = fs::read_to_string(&args.coverage_json).map_err(|error| {
        format!("could not read coverage JSON {}: {error}", args.coverage_json.display())
    })?;
    let allowlist =
        CoverageAllowlist::from_toml_str(&allowlist_text).map_err(|error| error.to_string())?;
    let report = evaluate_coverage_json(&coverage_json, &allowlist, &args.project_root)
        .map_err(|error| error.to_string())?;
    let report_text = report.to_text();

    if let Some(output) = args.output {
        fs::write(&output, &report_text)
            .map_err(|error| format!("could not write report {}: {error}", output.display()))?;
    }

    if report.is_passing() { Ok(report_text) } else { Err(report_text) }
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
    allowlist: PathBuf,
    coverage_json: PathBuf,
    project_root: PathBuf,
    output: Option<PathBuf>,
}

impl Args {
    fn parse(raw_args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut args = Self { project_root: PathBuf::from("."), ..Self::default() };
        let mut raw_args = raw_args.into_iter();

        while let Some(flag) = raw_args.next() {
            let value = raw_args.next().ok_or_else(|| format!("{flag} requires a value"))?;
            match flag.as_str() {
                "--allowlist" => args.allowlist = PathBuf::from(value),
                "--coverage-json" => args.coverage_json = PathBuf::from(value),
                "--project-root" => args.project_root = PathBuf::from(value),
                "--output" => args.output = Some(PathBuf::from(value)),
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        if args.allowlist.as_os_str().is_empty() {
            return Err("--allowlist is required".to_string());
        }
        if args.coverage_json.as_os_str().is_empty() {
            return Err("--coverage-json is required".to_string());
        }

        Ok(args)
    }
}
