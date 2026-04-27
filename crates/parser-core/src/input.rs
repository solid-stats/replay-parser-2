//! Parser input and option types.

/// Replay bytes and caller-provided metadata consumed by the pure parser core.
#[derive(Debug, Clone, PartialEq)]
pub struct ParserInput<'a> {
    /// Replay bytes to parse.
    pub bytes: &'a [u8],
    /// Replay source identity supplied by the caller.
    pub source: parser_contract::source_ref::ReplaySource,
    /// Parser identity supplied by the caller or adapter.
    pub parser: parser_contract::version::ParserInfo,
    /// Deterministic parser options.
    pub options: ParserOptions,
}

/// Deterministic parser options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParserOptions {
    /// Maximum number of diagnostics emitted before later plans collapse repeated issues.
    pub diagnostic_limit: usize,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self { diagnostic_limit: 100 }
    }
}
