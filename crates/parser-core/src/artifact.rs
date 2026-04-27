//! Parse artifact construction.

use std::collections::BTreeMap;

use parser_contract::{
    aggregates::AggregateSection,
    artifact::{ParseArtifact, ParseStatus},
    version::ContractVersion,
};

use crate::input::ParserInput;

/// Parses replay bytes into a deterministic artifact shell.
#[must_use]
pub fn parse_replay(input: ParserInput<'_>) -> ParseArtifact {
    let _diagnostic_limit = crate::diagnostics::DiagnosticPolicy::from(input.options).limit();

    ParseArtifact {
        contract_version: ContractVersion::current(),
        parser: input.parser,
        source: input.source,
        status: ParseStatus::Success,
        produced_at: None,
        diagnostics: Vec::new(),
        replay: None,
        entities: Vec::new(),
        events: Vec::new(),
        aggregates: AggregateSection::default(),
        failure: None,
        extensions: BTreeMap::new(),
    }
}
