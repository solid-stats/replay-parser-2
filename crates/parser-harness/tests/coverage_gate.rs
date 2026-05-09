//! Coverage allowlist policy tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use parser_harness::coverage::{CoverageAllowlist, CoverageAllowlistError, evaluate_coverage_json};

#[test]
fn coverage_gate_empty_allowlist_should_validate_without_exclusions() {
    // Arrange
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("empty allowlist should parse");
    let root = temp_project_root("empty_allowlist").expect("temp root should be created");

    // Act
    let result = allowlist.validate_against_root(&root);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn coverage_gate_allowlist_should_reject_exclusion_with_empty_reason() {
    // Arrange
    let allowlist = CoverageAllowlist::from_toml_str(
        r#"
[[exclusions]]
path = "crates/example/src/generated.rs"
pattern = "fallback"
lines = [7]
reason = " "
reviewer = "phase-05"
expires = "2026-05-28"
"#,
    )
    .expect("allowlist with empty reason should still parse");
    let root = temp_project_root("empty_reason").expect("temp root should be created");

    // Act
    let error =
        allowlist.validate_against_root(&root).expect_err("empty reason should fail validation");

    // Assert
    assert!(matches!(error, CoverageAllowlistError::EmptyField { field: "reason", .. }));
}

#[test]
fn coverage_gate_allowlist_should_reject_exclusion_without_exact_lines() {
    // Arrange
    let allowlist = CoverageAllowlist::from_toml_str(
        r#"
[[exclusions]]
path = "crates/example/src/generated.rs"
pattern = "fallback"
reason = "defensive unreachable branch kept for parser diagnostics"
reviewer = "phase-05"
expires = "2026-05-28"
"#,
    )
    .expect("allowlist without explicit lines should parse with default empty lines");
    let root = temp_project_root("empty_lines").expect("temp root should be created");

    // Act
    let error =
        allowlist.validate_against_root(&root).expect_err("empty lines should fail validation");

    // Assert
    assert!(matches!(error, CoverageAllowlistError::EmptyLines { .. }));
}

#[test]
fn coverage_gate_allowlist_should_reject_invalid_toml() {
    // Arrange + Act
    let error = CoverageAllowlist::from_toml_str("exclusions = ")
        .expect_err("invalid TOML should fail parsing");

    // Assert
    assert!(matches!(error, CoverageAllowlistError::InvalidToml { .. }));
}

#[test]
fn coverage_gate_allowlist_should_reject_blanket_non_generated_exclusion() {
    // Arrange
    let allowlist = CoverageAllowlist::from_toml_str(
        r#"
[[exclusions]]
path = "crates/example/src/module.rs"
pattern = "*"
lines = [7]
reason = "defensive fallback"
reviewer = "phase-05"
expires = "2026-05-28"
"#,
    )
    .expect("blanket exclusion should parse before validation");
    let root = temp_project_root("blanket_non_generated").expect("temp root should be created");

    // Act
    let error = allowlist
        .validate_against_root(&root)
        .expect_err("blanket non-generated exclusion should fail validation");

    // Assert
    assert!(matches!(error, CoverageAllowlistError::BlanketNonGenerated { .. }));
}

#[test]
fn coverage_gate_allowlist_should_reject_missing_target_file() {
    // Arrange
    let allowlist = CoverageAllowlist::from_toml_str(
        r#"
[[exclusions]]
path = "crates/example/src/missing.rs"
pattern = "defensive_fallback"
lines = [7]
reason = "defensive unreachable branch kept for parser diagnostics"
reviewer = "phase-05"
expires = "2026-05-28"
"#,
    )
    .expect("concrete exclusion should parse before target validation");
    let root = temp_project_root("missing_target").expect("temp root should be created");

    // Act
    let error = allowlist
        .validate_against_root(&root)
        .expect_err("missing target file should fail validation");

    // Assert
    assert!(matches!(error, CoverageAllowlistError::TargetRead { .. }));
}

#[test]
fn coverage_gate_allowlist_should_require_inline_marker_for_concrete_exclusion() {
    // Arrange
    let allowlist = CoverageAllowlist::from_toml_str(
        r#"
[[exclusions]]
path = "crates/example/src/module.rs"
pattern = "defensive_fallback"
lines = [7]
reason = "defensive unreachable branch kept for parser diagnostics"
reviewer = "phase-05"
expires = "2026-05-28"
"#,
    )
    .expect("concrete exclusion should parse before marker validation");
    let root = temp_project_root("missing_marker").expect("temp root should be created");
    write_project_file(&root, "crates/example/src/module.rs", "pub fn defensive_fallback() {}\n");

    // Act
    let error = allowlist
        .validate_against_root(&root)
        .expect_err("missing inline marker should fail validation");

    // Assert
    assert!(matches!(error, CoverageAllowlistError::MissingInlineMarker { .. }));
}

#[test]
fn coverage_gate_allowlist_should_accept_concrete_exclusion_with_inline_marker() {
    // Arrange
    let allowlist = CoverageAllowlist::from_toml_str(
        r#"
[[exclusions]]
path = "crates/example/src/module.rs"
pattern = "defensive_fallback"
lines = [7]
reason = "defensive unreachable branch kept for parser diagnostics"
reviewer = "phase-05"
expires = "2026-05-28"
"#,
    )
    .expect("concrete exclusion should parse before marker validation");
    let root = temp_project_root("with_marker").expect("temp root should be created");
    write_project_file(
        &root,
        "crates/example/src/module.rs",
        "// coverage-exclusion: defensive fallback is impossible with validated inputs\npub fn defensive_fallback() {}\n",
    );

    // Act
    let result = allowlist.validate_against_root(&root);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn coverage_gate_allowlist_should_accept_absolute_target_path_with_inline_marker() {
    // Arrange
    let root = temp_project_root("absolute_path").expect("temp root should be created");
    let absolute_path = root.join("generated.rs");
    fs::write(
        &absolute_path,
        "// coverage-exclusion: generated code wrapper is not reachable in tests\n",
    )
    .expect("absolute target should be written");
    let allowlist = CoverageAllowlist::from_toml_str(&format!(
        r#"
[[exclusions]]
path = "{}"
pattern = "*"
lines = [1]
reason = "generated code wrapper"
reviewer = "phase-05"
expires = "2026-05-28"
"#,
        absolute_path.display()
    ))
    .expect("absolute exclusion should parse before marker validation");

    // Act
    let result = allowlist.validate_against_root(&root);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn coverage_gate_report_should_fail_when_uncovered_production_line_is_not_allowlisted() {
    // Arrange
    let root = temp_project_root("report_missing_allowlist").expect("temp root should be created");
    write_project_file(
        &root,
        "crates/example/src/lib.rs",
        "// coverage-exclusion: defensive fallback is unreachable\npub fn fallback() {}\n",
    );
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json = coverage_json_for_file(&root, "crates/example/src/lib.rs", 2, 0);

    // Act
    let report = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect("coverage JSON should evaluate");

    // Assert
    assert!(!report.is_passing());
    assert_eq!(report.uncovered_locations.len(), 1);
    assert_eq!(report.uncovered_locations[0].line, 2);
    assert_eq!(
        report.to_text(),
        "production_files=1\nallowlisted_locations=0\nuncovered_locations=1\nuncovered:\ncrates/example/src/lib.rs:2 region\n"
    );
}

#[test]
fn coverage_gate_report_should_pass_when_uncovered_production_line_is_allowlisted() {
    // Arrange
    let root = temp_project_root("report_allowlisted").expect("temp root should be created");
    write_project_file(
        &root,
        "crates/example/src/lib.rs",
        "// coverage-exclusion: defensive fallback is unreachable\npub fn fallback() {}\n",
    );
    let allowlist = CoverageAllowlist::from_toml_str(
        r#"
[[exclusions]]
path = "crates/example/src/lib.rs"
pattern = "fallback"
lines = [2]
reason = "defensive unreachable branch kept for parser diagnostics"
reviewer = "phase-05"
expires = "2026-05-28"
"#,
    )
    .expect("allowlist should parse");
    let coverage_json = coverage_json_for_file(&root, "crates/example/src/lib.rs", 2, 0);

    // Act
    let report = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect("coverage JSON should evaluate");

    // Assert
    assert!(report.is_passing());
    assert_eq!(report.allowlisted_locations, 1);
}

#[test]
fn coverage_gate_report_should_ignore_source_unit_test_module_lines() {
    // Arrange
    let root =
        temp_project_root("report_ignores_source_unit_tests").expect("temp root should be created");
    write_project_file(
        &root,
        "crates/example/src/lib.rs",
        r#"pub fn production_path() {}

#[cfg(test)]
mod tests {
    #[test]
    fn source_level_test_helper() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
    );
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json =
        coverage_json_for_segments(&root, "crates/example/src/lib.rs", &[(1, 0), (6, 0)]);

    // Act
    let report = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect("coverage JSON should evaluate");

    // Assert
    assert_eq!(report.uncovered_locations.len(), 1);
    assert_eq!(report.uncovered_locations[0].line, 1);
}

#[test]
fn coverage_gate_report_should_ignore_source_unit_test_module_after_outer_attribute_and_blank_line()
{
    // Arrange
    let root = temp_project_root("report_ignores_source_unit_tests_with_attribute")
        .expect("temp root should be created");
    write_project_file(
        &root,
        "crates/example/src/lib.rs",
        r#"pub fn production_path() {}

#[cfg(test)]
#[allow(clippy::expect_used)]

mod tests {
    #[test]
    fn source_level_test_helper() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
    );
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json =
        coverage_json_for_segments(&root, "crates/example/src/lib.rs", &[(1, 1), (8, 0)]);

    // Act
    let report = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect("coverage JSON should evaluate");

    // Assert
    assert!(report.is_passing());
    assert!(report.uncovered_locations.is_empty());
}

#[test]
fn coverage_gate_report_should_reject_json_without_data_array() {
    // Arrange
    let root = temp_project_root("report_missing_data").expect("temp root should be created");
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");

    // Act
    let error = evaluate_coverage_json(r#"{"not_data":[]}"#, &allowlist, &root)
        .expect_err("coverage JSON without data[0] should fail");

    // Assert
    assert!(matches!(
        error,
        CoverageAllowlistError::CoverageJsonShape { message: "missing data[0]" }
    ));
}

#[test]
fn coverage_gate_report_should_reject_data_entry_without_files() {
    // Arrange
    let root = temp_project_root("report_missing_files").expect("temp root should be created");
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");

    // Act
    let error = evaluate_coverage_json(r#"{"data":[{"files":null}]}"#, &allowlist, &root)
        .expect_err("coverage JSON without files should fail");

    // Assert
    assert!(matches!(
        error,
        CoverageAllowlistError::CoverageJsonShape { message: "missing files" }
    ));
}

#[test]
fn coverage_gate_report_should_reject_file_without_segments() {
    // Arrange
    let root = temp_project_root("report_missing_segments").expect("temp root should be created");
    write_project_file(&root, "crates/example/src/lib.rs", "pub fn production_path() {}\n");
    let filename = root.join("crates/example/src/lib.rs");
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json = format!(
        r#"{{
  "data": [
    {{
      "files": [{{ "filename": "{}", "segments": null }}]
    }}
  ]
}}"#,
        filename.display()
    );

    // Act
    let error = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect_err("coverage JSON file without segments should fail");

    // Assert
    assert!(matches!(
        error,
        CoverageAllowlistError::CoverageJsonShape { message: "missing file segments" }
    ));
}

#[test]
fn coverage_gate_report_should_reject_non_array_segment() {
    // Arrange
    let root = temp_project_root("report_non_array_segment").expect("temp root should be created");
    write_project_file(&root, "crates/example/src/lib.rs", "pub fn production_path() {}\n");
    let filename = root.join("crates/example/src/lib.rs");
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json = format!(
        r#"{{
  "data": [
    {{
      "files": [
        {{
          "filename": "{}",
          "segments": [{{ "line": 1 }}]
        }}
      ]
    }}
  ]
}}"#,
        filename.display()
    );

    // Act
    let error = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect_err("coverage JSON with non-array segment should fail");

    // Assert
    assert!(matches!(
        error,
        CoverageAllowlistError::CoverageJsonShape { message: "segment is not an array" }
    ));
}

#[test]
fn coverage_gate_report_should_reject_uncovered_segment_without_u32_line() {
    // Arrange
    let root = temp_project_root("report_bad_segment_line").expect("temp root should be created");
    write_project_file(&root, "crates/example/src/lib.rs", "pub fn production_path() {}\n");
    let filename = root.join("crates/example/src/lib.rs");
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json = format!(
        r#"{{
  "data": [
    {{
      "files": [
        {{
          "filename": "{}",
          "segments": [["line-one", 1, 0, true, true]]
        }}
      ]
    }}
  ]
}}"#,
        filename.display()
    );

    // Act
    let error = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect_err("coverage JSON with non-u32 segment line should fail");

    // Assert
    assert!(matches!(
        error,
        CoverageAllowlistError::CoverageJsonShape { message: "segment line is not a u32" }
    ));
}

#[test]
fn coverage_gate_report_should_ignore_non_counted_and_covered_segments() {
    // Arrange
    let root =
        temp_project_root("report_ignores_inactive_segments").expect("temp root should be created");
    write_project_file(
        &root,
        "crates/example/src/lib.rs",
        "pub fn first() {}\npub fn second() {}\n",
    );
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json = coverage_json_for_custom_segments(
        &root,
        "crates/example/src/lib.rs",
        &["[1, 1, 0, false, true]", "[2, 1, 7, true, true]"],
    );

    // Act
    let report = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect("coverage JSON should evaluate");

    // Assert
    assert!(report.is_passing());
    assert!(report.uncovered_locations.is_empty());
}

#[test]
fn coverage_gate_report_should_ignore_segments_without_count_metadata() {
    // Arrange
    let root = temp_project_root("report_ignores_segments_without_counts")
        .expect("temp root should be created");
    write_project_file(&root, "crates/example/src/lib.rs", "pub fn production_path() {}\n");
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json = coverage_json_for_custom_segments(
        &root,
        "crates/example/src/lib.rs",
        &["[1, 1, 0]", "[1, 3, 0, false, true]"],
    );

    // Act
    let report = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect("coverage JSON should evaluate");

    // Assert
    assert!(report.is_passing());
    assert!(report.uncovered_locations.is_empty());
}

#[test]
fn coverage_gate_report_should_not_treat_covered_line_subregion_as_uncovered_line() {
    // Arrange
    let root = temp_project_root("report_ignores_covered_line_subregions")
        .expect("temp root should be created");
    write_project_file(
        &root,
        "crates/example/src/lib.rs",
        "pub fn branchy(value: bool) { if value { println!(\"yes\"); } }\n",
    );
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json = coverage_json_for_custom_segments(
        &root,
        "crates/example/src/lib.rs",
        &["[1, 1, 1, true, true]", "[1, 31, 0, true, true]"],
    );

    // Act
    let report = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect("coverage JSON should evaluate");

    // Assert
    assert!(report.is_passing());
    assert!(report.uncovered_locations.is_empty());
}

#[test]
fn coverage_gate_report_should_ignore_files_outside_production_sources() {
    // Arrange
    let root = temp_project_root("report_ignores_non_production_files")
        .expect("temp root should be created");
    write_project_file(&root, "crates/example/tests/integration.rs", "fn helper() {}\n");
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json = coverage_json_for_file(&root, "crates/example/tests/integration.rs", 1, 0);

    // Act
    let report = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect("coverage JSON should evaluate");

    // Assert
    assert_eq!(report.production_files, 0);
    assert!(report.is_passing());
}

#[test]
fn coverage_gate_report_should_reject_missing_production_source_file() {
    // Arrange
    let root =
        temp_project_root("report_missing_production_source").expect("temp root should be created");
    let allowlist =
        CoverageAllowlist::from_toml_str("exclusions = []").expect("allowlist should parse");
    let coverage_json = coverage_json_for_file(&root, "crates/example/src/missing.rs", 1, 0);

    // Act
    let error = evaluate_coverage_json(&coverage_json, &allowlist, &root)
        .expect_err("missing production source file should fail evaluation");

    // Assert
    assert!(matches!(error, CoverageAllowlistError::TargetRead { .. }));
}

fn temp_project_root(test_name: &str) -> std::io::Result<PathBuf> {
    let root = std::env::temp_dir()
        .join(format!("replay-parser-2-coverage-gate-{}-{test_name}", std::process::id()));

    match fs::remove_dir_all(&root) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    fs::create_dir_all(&root)?;
    Ok(root)
}

fn write_project_file(root: &Path, relative_path: &str, contents: &str) {
    let path = root.join(relative_path);
    let parent = path.parent().expect("test file should have a parent directory");
    fs::create_dir_all(parent).expect("test file parent directory should be created");
    fs::write(path, contents).expect("test file should be written");
}

fn coverage_json_for_file(root: &Path, relative_path: &str, line: u32, count: u32) -> String {
    coverage_json_for_segments(root, relative_path, &[(line, count)])
}

fn coverage_json_for_segments(root: &Path, relative_path: &str, segments: &[(u32, u32)]) -> String {
    let segments = segments
        .iter()
        .map(|(line, count)| format!("[{line}, 1, {count}, true, true]"))
        .collect::<Vec<_>>();
    coverage_json_for_custom_segments(root, relative_path, &segments)
}

fn coverage_json_for_custom_segments(
    root: &Path,
    relative_path: &str,
    segments: &[impl AsRef<str>],
) -> String {
    let filename = root.join(relative_path);
    let segments = segments.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(", ");
    format!(
        r#"{{
  "data": [
    {{
      "files": [
        {{
          "filename": "{}",
          "segments": [{}]
        }}
      ],
      "functions": [
        {{
          "count": 1,
          "filenames": ["{}"],
          "regions": [[1, 1, 1, 10, 1, 0, 0, 0]]
        }}
      ]
    }}
  ]
}}"#,
        filename.display(),
        segments,
        filename.display(),
    )
}
