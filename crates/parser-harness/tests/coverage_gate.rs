//! Coverage allowlist policy tests.

#![allow(
    clippy::expect_used,
    reason = "integration tests use expect messages as assertion context"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use parser_harness::coverage::{CoverageAllowlist, CoverageAllowlistError};

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
