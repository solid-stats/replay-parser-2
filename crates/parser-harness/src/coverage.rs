//! Coverage allowlist parsing and validation helpers.
//!
//! Exclusions are review exceptions for unreachable/generated/defensive code,
//! not a substitute for behavior-level tests.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

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
        validate_non_empty(&self.expires, "expires", &self.path)
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
}
