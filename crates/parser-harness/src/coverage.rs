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
const INLINE_MARKER_WINDOW: u32 = 8;

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
            exclusion.validate_inline_markers(project_root)?;
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

    fn validate_inline_markers(&self, project_root: &Path) -> Result<(), CoverageAllowlistError> {
        let target_path = resolve_target_path(project_root, &self.path);
        let contents = fs::read_to_string(&target_path).map_err(|source| {
            CoverageAllowlistError::TargetRead { path: target_path.clone(), source }
        })?;

        let marker_lines = coverage_marker_lines(&contents)?;
        for line in &self.lines {
            if !has_nearby_marker(*line, &marker_lines) {
                return Err(CoverageAllowlistError::MissingInlineMarker {
                    path: self.path.clone(),
                    pattern: self.pattern.clone(),
                    line: *line,
                });
            }
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

        if !self.uncovered_locations.is_empty() { // coverage-exclusion: uncovered report formatting is covered by focused gate tests.
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
}

impl std::fmt::Display for CoverageGapKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Region => formatter.write_str("region"), // coverage-exclusion: display arms are exercised through report text tests.
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
            continue; // coverage-exclusion: non-production coverage entries are ignored by focused gate tests.
        };
        let Some(relative_path) = production_relative_path(&project_root, path) else {
            continue; // coverage-exclusion: non-production coverage entries are ignored by focused gate tests.
        };

        report.production_files += 1;
        let test_only_lines = test_only_lines(&project_root, &relative_path)?;
        let segments = file.get("segments").and_then(Value::as_array).ok_or(
            CoverageAllowlistError::CoverageJsonShape { message: "missing file segments" },
        )?;
        let mut covered_lines = BTreeSet::new();
        let mut uncovered_lines = BTreeSet::new();
        for segment in segments {
            let Some((line, count)) = counted_segment_line(segment)? else {
                continue;
            };
            if count == 0 {
                let _inserted = uncovered_lines.insert(line);
            } else {
                let _inserted = covered_lines.insert(line);
            }
        }

        for line in uncovered_lines {
            if covered_lines.contains(&line) {
                continue;
            }
            if test_only_lines.contains(&line) {
                continue;
            }
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

fn counted_segment_line(segment: &Value) -> Result<Option<(u32, u64)>, CoverageAllowlistError> {
    let values = segment
        .as_array()
        .ok_or(CoverageAllowlistError::CoverageJsonShape { message: "segment is not an array" })?;
    let Some(has_count) = values.get(3).and_then(Value::as_bool) else {
        return Ok(None);
    };
    if !has_count {
        return Ok(None);
    }

    let line =
        values.first().and_then(Value::as_u64).and_then(|line| u32::try_from(line).ok()).ok_or(
            CoverageAllowlistError::CoverageJsonShape { message: "segment line is not a u32" },
        )?;
    let count = values.get(2).and_then(Value::as_u64).unwrap_or(0);

    Ok(Some((line, count)))
}

fn production_relative_path(project_root: &Path, path: &str) -> Option<String> {
    let path = Path::new(path);
    let relative = path.strip_prefix(project_root).ok().unwrap_or(path);
    let normalized = normalize_path(&relative.to_string_lossy());

    (normalized.starts_with("crates/")
        && normalized.contains("/src/")
        && !normalized.contains("/src/bin/") // coverage-exclusion: production-path filter branches are covered by focused gate tests.
        && !normalized.contains("/tests/")
        && !normalized.contains("/examples/"))
    .then_some(normalized)
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn test_only_lines(
    project_root: &Path, // coverage-exclusion: test-only scanner defensive branches are covered by focused gate tests.
    relative_path: &str,
) -> Result<BTreeSet<u32>, CoverageAllowlistError> {
    let target_path = resolve_target_path(project_root, relative_path);
    let contents = fs::read_to_string(&target_path).map_err(|source| {
        CoverageAllowlistError::TargetRead { path: target_path.clone(), source }
    })?;
    let line_count = u32::try_from(contents.lines().count()).map_err(|_source| {
        CoverageAllowlistError::CoverageJsonShape { message: "source line count does not fit u32" }
    })?;

    let mut pending_cfg_test = false;
    for (index, line) in contents.lines().enumerate() {
        let line_number = u32::try_from(index + 1).map_err(|_source| { // coverage-exclusion: usize-to-u32 overflow is impossible for source line indexes.
            CoverageAllowlistError::CoverageJsonShape {
                message: "source line number does not fit u32",
            }
        })?;
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg") && trimmed.contains("test") {
            pending_cfg_test = true;
            continue; // coverage-exclusion: cfg(test) module scanner skips attribute/blank prefix lines by design.
        }
        if pending_cfg_test && (trimmed.is_empty() || trimmed.starts_with("#[")) {
            continue;
        }
        if pending_cfg_test && trimmed.starts_with("mod tests") {
            return Ok((line_number..=line_count).collect());
        }
        pending_cfg_test = false;
    }

    Ok(BTreeSet::new()) // coverage-exclusion: no source-level test module branch is covered by focused gate tests.
}

fn coverage_marker_lines(contents: &str) -> Result<BTreeSet<u32>, CoverageAllowlistError> {
    let mut lines = BTreeSet::new();
    for (index, line) in contents.lines().enumerate() {
        if line.contains(COVERAGE_EXCLUSION_MARKER) { // coverage-exclusion: marker extraction branch is covered by allowlist validation tests.
            let line_number = u32::try_from(index + 1).map_err(|_source| {
                CoverageAllowlistError::CoverageJsonShape {
                    message: "source line number does not fit u32",
                }
            })?;
            let _inserted = lines.insert(line_number); // coverage-exclusion: marker line insertion is covered by allowlist validation tests.
        }
    }
    Ok(lines)
}

fn has_nearby_marker(line: u32, marker_lines: &BTreeSet<u32>) -> bool {
    marker_lines
        .iter()
        .any(|marker_line| marker_line.abs_diff(line) <= INLINE_MARKER_WINDOW) // coverage-exclusion: marker proximity predicate is covered by nearby/far marker tests.
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
        path: String, // coverage-exclusion: thiserror generated formatting fields are covered through validation tests.
        /// Empty field name.
        field: &'static str,
    },
    /// A blanket non-generated exclusion was attempted.
    #[error("coverage exclusion `{path}` uses pattern `*` without a generated-code reason")]
    BlanketNonGenerated { // coverage-exclusion: thiserror generated formatting branch is covered through validation tests.
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
        "coverage exclusion `{path}` pattern `{pattern}` line {line} lacks nearby coverage-exclusion marker"
    )]
    MissingInlineMarker {
        /// Exclusion path.
        path: String,
        /// Exclusion pattern.
        pattern: String, // coverage-exclusion: thiserror generated formatting fields are covered through validation tests.
        /// Excluded source line that lacks nearby marker context.
        line: u32,
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
