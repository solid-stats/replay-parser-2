use parser_contract::schema::parse_artifact_schema;

fn main() -> Result<(), serde_json::Error> {
    let schema = parse_artifact_schema();
    let schema_text = serde_json::to_string_pretty(&schema)?;
    println!("{schema_text}");
    Ok(())
}
