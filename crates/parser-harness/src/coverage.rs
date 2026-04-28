//! Coverage allowlist parsing and validation helpers.
//!
//! Exclusions are review exceptions for unreachable/generated/defensive code,
//! not a substitute for behavior-level tests.
// coverage-exclusion: reviewed Phase 05 coverage-check defensive branches are allowlisted by exact source line.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Inline marker required near code covered by an allowlist exclusion.
pub const COVERAGE_EXCLUSION_MARKER: &str = "coverage-exclusion:";

/// Reviewable coverage allowlist document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageAllowlist {
    /// Narrow coverage exclusions approved for production code.
    #[serde(default)]
    pub exclusions: Vec<CoverageExclusion>,
}

impl CoverageAllowlist {
    /// Parses a coverage allowlist from TOML.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageAllowlistError::InvalidToml`] when the document is not
    /// a valid allowlist.
    pub fn from_toml_str(source: &str) -> Result<Self, CoverageAllowlistError> {
        toml::from_str(source).map_err(|source| CoverageAllowlistError::InvalidToml { source })
    }

    /// Validates exclusion policy against production files under `project_root`.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageAllowlistError`] when an exclusion is too broad, lacks
    /// required review metadata, or has no inline marker in the target file.
    pub fn validate_against_root(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<(), CoverageAllowlistError> {
        let project_root = project_root.as_ref();

        for exclusion in &self.exclusions {
            exclusion.validate_metadata()?;
            exclusion.validate_blanket_scope()?;
            exclusion.validate_inline_marker(project_root)?;
        }

        Ok(())
    }
}

/// A single allowlisted coverage exclusion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageExclusion {
    /// Production file path relative to the repository root.
    pub path: String,
    /// Specific branch/function/pattern excluded from the coverage gate.
    pub pattern: String,
    /// Exact 1-based source lines covered by this exclusion.
    #[serde(default)]
    pub lines: Vec<u32>,
    /// Review rationale explaining why tests cannot reasonably cover this code.
    pub reason: String,
    /// Reviewer or approval reference.
    pub reviewer: String,
    /// Expiration date or review deadline.
    pub expires: String,
}

impl CoverageExclusion {
    fn validate_metadata(&self) -> Result<(), CoverageAllowlistError> {
        validate_non_empty(&self.path, "path", &self.path)?;
        validate_non_empty(&self.pattern, "pattern", &self.path)?;
        validate_non_empty(&self.reason, "reason", &self.path)?;
        validate_non_empty(&self.reviewer, "reviewer", &self.path)?;
        validate_non_empty(&self.expires, "expires", &self.path)?;
        if self.lines.is_empty() {
            return Err(CoverageAllowlistError::EmptyLines { path: self.path.clone() });
        }

        Ok(())
    }

    fn validate_blanket_scope(&self) -> Result<(), CoverageAllowlistError> {
        if self.pattern.trim() == "*" && !self.reason.to_ascii_lowercase().contains("generated") {
            return Err(CoverageAllowlistError::BlanketNonGenerated { path: self.path.clone() });
        }

        Ok(())
    }

    fn validate_inline_marker(&self, project_root: &Path) -> Result<(), CoverageAllowlistError> {
        let target_path = resolve_target_path(project_root, &self.path);
        let contents = fs::read_to_string(&target_path).map_err(|source| {
            CoverageAllowlistError::TargetRead { path: target_path.clone(), source }
        })?;

        if !contents.contains(COVERAGE_EXCLUSION_MARKER) {
            return Err(CoverageAllowlistError::MissingInlineMarker {
                path: self.path.clone(),
                pattern: self.pattern.clone(),
            });
        }

        Ok(())
    }
}

/// A parsed coverage gate evaluation result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CoverageGateReport {
    /// Production files inspected from the llvm-cov JSON report.
    pub production_files: usize,
    /// Uncovered production source locations that are allowlisted.
    pub allowlisted_locations: usize,
    /// Uncovered production source locations that are not allowlisted.
    pub uncovered_locations: Vec<CoverageGap>,
}

impl CoverageGateReport {
    /// Returns `true` when no unallowlisted production coverage gaps remain.
    #[must_use]
    pub const fn is_passing(&self) -> bool {
        self.uncovered_locations.is_empty()
    }

    /// Formats a stable human-readable report for script output.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("production_files={}", self.production_files),
            format!("allowlisted_locations={}", self.allowlisted_locations),
            format!("uncovered_locations={}", self.uncovered_locations.len()),
        ];

        if !self.uncovered_locations.is_empty() {
            lines.push("uncovered:".to_string());
            for gap in &self.uncovered_locations {
                lines.push(format!("{}:{} {}", gap.path, gap.line, gap.kind));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }
}

/// One uncovered production source location after allowlist filtering.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoverageGap {
    /// Repository-relative path.
    pub path: String,
    /// 1-based source line.
    pub line: u32,
    /// Gap kind emitted by the post-processor.
    pub kind: CoverageGapKind,
}

/// Coverage gap type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoverageGapKind {
    /// An uncovered source execution region.
    Region,
    /// An uncovered function start.
    Function,
}

impl std::fmt::Display for CoverageGapKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Region => formatter.write_str("region"),
            Self::Function => formatter.write_str("function"),
        }
    }
}

/// Evaluates `cargo llvm-cov --json` output against a narrow source-line allowlist.
///
/// # Errors
///
/// Returns [`CoverageAllowlistError::InvalidCoverageJson`] when the report is
/// not valid llvm-cov JSON or contains an unsupported shape.
pub fn evaluate_coverage_json(
    coverage_json: &str,
    allowlist: &CoverageAllowlist,
    project_root: impl AsRef<Path>,
) -> Result<CoverageGateReport, CoverageAllowlistError> {
    allowlist.validate_against_root(&project_root)?;

    let project_root = fs::canonicalize(project_root.as_ref())
        .unwrap_or_else(|_| project_root.as_ref().to_path_buf());
    let allowed_lines = allowlisted_lines(allowlist);
    let coverage: Value = serde_json::from_str(coverage_json)
        .map_err(|source| CoverageAllowlistError::InvalidCoverageJson { source })?;
    let data = coverage
        .get("data")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .ok_or(CoverageAllowlistError::CoverageJsonShape { message: "missing data[0]" })?;

    let mut report = CoverageGateReport::default();
    let mut gaps = BTreeSet::<CoverageGap>::new();

    for file in data
        .get("files")
        .and_then(Value::as_array)
        .ok_or(CoverageAllowlistError::CoverageJsonShape { message: "missing files" })?
    {
        let Some(path) = file.get("filename").and_then(Value::as_str) else {
            continue;
        };
        let Some(relative_path) = production_relative_path(&project_root, path) else {
            continue;
        };

        report.production_files += 1;
        let segments = file.get("segments").and_then(Value::as_array).ok_or(
            CoverageAllowlistError::CoverageJsonShape { message: "missing file segments" },
        )?;
        for segment in segments {
            let Some(line) = uncovered_segment_line(segment)? else {
                continue;
            };
            insert_or_allow_gap(
                &mut report,
                &mut gaps,
                &allowed_lines,
                &relative_path,
                line,
                CoverageGapKind::Region,
            );
        }
    }

    report.uncovered_locations = gaps.into_iter().collect();
    Ok(report)
}

fn allowlisted_lines(allowlist: &CoverageAllowlist) -> BTreeMap<String, BTreeSet<u32>> {
    let mut allowed = BTreeMap::<String, BTreeSet<u32>>::new();
    for exclusion in &allowlist.exclusions {
        let path = normalize_path(&exclusion.path);
        allowed.entry(path).or_default().extend(exclusion.lines.iter().copied());
    }
    allowed
}

fn insert_or_allow_gap(
    report: &mut CoverageGateReport,
    gaps: &mut BTreeSet<CoverageGap>,
    allowed_lines: &BTreeMap<String, BTreeSet<u32>>,
    relative_path: &str,
    line: u32,
    kind: CoverageGapKind,
) {
    if allowed_lines.get(relative_path).is_some_and(|lines| lines.contains(&line)) {
        report.allowlisted_locations += 1;
    } else {
        let _inserted = gaps.insert(CoverageGap { path: relative_path.to_string(), line, kind });
    }
}

fn uncovered_segment_line(segment: &Value) -> Result<Option<u32>, CoverageAllowlistError> {
    let values = segment
        .as_array()
        .ok_or(CoverageAllowlistError::CoverageJsonShape { message: "segment is not an array" })?;
    let Some(has_count) = values.get(3).and_then(Value::as_bool) else {
        return Ok(None);
    };
    let count = values.get(2).and_then(Value::as_u64).unwrap_or(0);
    if !has_count || count != 0 {
        return Ok(None);
    }

    values
        .first()
        .and_then(Value::as_u64)
        .and_then(|line| u32::try_from(line).ok())
        .map(Some)
        .ok_or(CoverageAllowlistError::CoverageJsonShape { message: "segment line is not a u32" })
}

fn production_relative_path(project_root: &Path, path: &str) -> Option<String> {
    let path = Path::new(path);
    let relative = path.strip_prefix(project_root).ok().unwrap_or(path);
    let normalized = normalize_path(&relative.to_string_lossy());

    (normalized.starts_with("crates/")
        && normalized.contains("/src/")
        && !normalized.contains("/src/bin/")
        && !normalized.contains("/tests/")
        && !normalized.contains("/examples/"))
    .then_some(normalized)
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn validate_non_empty(
    value: &str,
    field: &'static str,
    path: &str,
) -> Result<(), CoverageAllowlistError> {
    if value.trim().is_empty() {
        return Err(CoverageAllowlistError::EmptyField { path: path.to_owned(), field });
    }

    Ok(())
}

fn resolve_target_path(project_root: &Path, path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return candidate.to_path_buf();
    }

    project_root.join(candidate)
}

/// Coverage allowlist validation failures.
#[derive(Debug, thiserror::Error)]
pub enum CoverageAllowlistError {
    /// The allowlist was not valid TOML for [`CoverageAllowlist`].
    #[error("coverage allowlist TOML is invalid: {source}")]
    InvalidToml {
        /// TOML parser source error.
        source: toml::de::Error,
    },
    /// A required exclusion metadata field was empty.
    #[error("coverage exclusion `{path}` has empty {field}")]
    EmptyField {
        /// Exclusion path.
        path: String,
        /// Empty field name.
        field: &'static str,
    },
    /// A blanket non-generated exclusion was attempted.
    #[error("coverage exclusion `{path}` uses pattern `*` without a generated-code reason")]
    BlanketNonGenerated {
        /// Exclusion path.
        path: String,
    },
    /// A concrete exclusion did not list any exact source lines.
    #[error("coverage exclusion `{path}` has no exact lines")]
    EmptyLines {
        /// Exclusion path.
        path: String,
    },
    /// The referenced production file could not be read.
    #[error("coverage exclusion target `{path}` cannot be read: {source}")]
    TargetRead {
        /// Target file path.
        path: PathBuf,
        /// Filesystem source error.
        source: std::io::Error,
    },
    /// The target production file lacks an inline rationale marker.
    #[error(
        "coverage exclusion `{path}` pattern `{pattern}` lacks inline coverage-exclusion marker"
    )]
    MissingInlineMarker {
        /// Exclusion path.
        path: String,
        /// Exclusion pattern.
        pattern: String,
    },
    /// The generated coverage JSON could not be parsed.
    #[error("coverage JSON is invalid: {source}")]
    InvalidCoverageJson {
        /// JSON parser source error.
        source: serde_json::Error,
    },
    /// The generated coverage JSON did not match the expected llvm-cov shape.
    #[error("coverage JSON has unsupported shape: {message}")]
    CoverageJsonShape {
        /// Static shape failure description.
        message: &'static str,
    },
}
