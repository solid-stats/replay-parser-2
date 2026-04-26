//! Exports the current parse artifact JSON Schema to stdout.

use parser_contract::schema::parse_artifact_schema;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = parse_artifact_schema();
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &schema)?;
    stdout.write_all(b"\n")?;
    Ok(())
}
