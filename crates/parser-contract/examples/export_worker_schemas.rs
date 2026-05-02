//! Exports the parser-worker JSON Schemas to an output directory.

use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use parser_contract::schema::{parse_job_schema, parse_result_schema};
use schemars::Schema;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = output_dir_arg()?;
    fs::create_dir_all(&output_dir)?;
    write_schema(&output_dir.join("parse-job-v1.schema.json"), &parse_job_schema())?;
    write_schema(&output_dir.join("parse-result-v1.schema.json"), &parse_result_schema())?;
    Ok(())
}

fn output_dir_arg() -> Result<PathBuf, io::Error> {
    let mut args = env::args_os();
    drop(args.next());

    let Some(flag) = args.next() else {
        return Err(invalid_args());
    };
    if flag != "--output-dir" {
        return Err(invalid_args());
    }

    let Some(output_dir) = args.next() else {
        return Err(invalid_args());
    };
    if args.next().is_some() {
        return Err(invalid_args());
    }

    Ok(PathBuf::from(output_dir))
}

fn invalid_args() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "usage: export_worker_schemas --output-dir <dir>")
}

fn write_schema(path: &Path, schema: &Schema) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;
    serde_json::to_writer_pretty(&mut file, schema)?;
    file.write_all(b"\n")?;
    Ok(())
}
