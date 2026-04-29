//! Human-readable summary rendering for comparison reports.

use std::{collections::BTreeMap, fmt::Write as _};

use crate::report::{ComparisonReport, ImpactLevel, MismatchCategory};

/// A high-priority comparison diff for human review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonTopDiff {
    /// Compared compact artifact surface.
    pub surface: String,
    /// Mismatch category assigned to this diff.
    pub category: MismatchCategory,
    /// Whether the parser artifact differs.
    pub parser_artifact_impact: ImpactLevel,
    /// Whether `server-2` recalculation can differ.
    pub server_2_recalculation_impact: ImpactLevel,
    /// Whether public stats through `web` can differ.
    pub ui_visible_public_stats_impact: ImpactLevel,
    /// Reviewer note or next action for the diff.
    pub note: String,
}

/// Summary-first comparison review model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonReviewSummary {
    /// Number of findings in the source report.
    pub total_findings: usize,
    /// Finding counts keyed by mismatch category string.
    pub by_category: BTreeMap<String, usize>,
    /// Finding counts keyed by impact dimension and value.
    pub by_impact: BTreeMap<String, usize>,
    /// Up to ten prioritized diffs requiring the most attention.
    pub top_diffs: Vec<ComparisonTopDiff>,
    /// Report-level next action.
    pub next_action: String,
}

impl ComparisonReviewSummary {
    /// Builds a human-review summary from a structured report.
    #[must_use]
    pub fn from_report(report: &ComparisonReport) -> Self {
        let mut by_impact = BTreeMap::new();
        for finding in &report.findings {
            increment_impact(&mut by_impact, "parser_artifact", finding.impact.parser_artifact);
            increment_impact(
                &mut by_impact,
                "server_2_persistence",
                finding.impact.server_2_persistence,
            );
            increment_impact(
                &mut by_impact,
                "server_2_recalculation",
                finding.impact.server_2_recalculation,
            );
            increment_impact(
                &mut by_impact,
                "ui_visible_public_stats",
                finding.impact.ui_visible_public_stats,
            );
        }

        let mut prioritized_findings = report.findings.iter().collect::<Vec<_>>();
        prioritized_findings.sort_by_key(|finding| category_priority(finding.category));
        let top_diffs = prioritized_findings
            .into_iter()
            .take(10)
            .map(|finding| ComparisonTopDiff {
                surface: finding.surface.clone(),
                category: finding.category,
                parser_artifact_impact: finding.impact.parser_artifact,
                server_2_recalculation_impact: finding.impact.server_2_recalculation,
                ui_visible_public_stats_impact: finding.impact.ui_visible_public_stats,
                note: note_for_finding(finding.category, &finding.notes),
            })
            .collect();

        let next_action = if report
            .findings
            .iter()
            .any(|finding| finding.category == MismatchCategory::HumanReview)
        {
            "Review top human_review diffs before accepting parity.".to_owned()
        } else if report.findings.iter().any(|finding| {
            matches!(
                finding.category,
                MismatchCategory::NewBug | MismatchCategory::InsufficientData
            )
        }) {
            "Resolve blocking comparison findings before accepting parity.".to_owned()
        } else {
            "No blocking comparison review action recorded.".to_owned()
        };

        Self {
            total_findings: report.findings.len(),
            by_category: report.summary.by_category.clone(),
            by_impact,
            top_diffs,
            next_action,
        }
    }
}

/// Renders a summary-first Markdown comparison report.
#[must_use]
pub fn render_markdown_summary(report: &ComparisonReport) -> String {
    let summary = ComparisonReviewSummary::from_report(report);
    let mut markdown = String::new();

    markdown.push_str("# Comparison Summary\n\n");
    let _ = write!(markdown, "Total findings: {}\n\n", summary.total_findings);

    markdown.push_str("## Counts by Category\n\n");
    push_count_lines(&mut markdown, &summary.by_category);

    markdown.push_str("\n## Counts by Impact\n\n");
    push_count_lines(&mut markdown, &summary.by_impact);

    markdown.push_str("\n## Top Diffs\n\n");
    if summary.top_diffs.is_empty() {
        markdown.push_str("- None\n");
    } else {
        for diff in summary.top_diffs {
            let _ = writeln!(
                markdown,
                "- `{}`: `{}`; parser_artifact={}; server_2_recalculation={}; ui_visible_public_stats={}; note={}",
                diff.surface,
                diff.category,
                impact_str(diff.parser_artifact_impact),
                impact_str(diff.server_2_recalculation_impact),
                impact_str(diff.ui_visible_public_stats_impact),
                diff.note
            );
        }
    }

    markdown.push_str("\n## Next Action\n\n");
    markdown.push_str(&summary.next_action);
    markdown.push('\n');

    markdown
}

fn increment_impact(counts: &mut BTreeMap<String, usize>, dimension: &str, impact: ImpactLevel) {
    let count = counts.entry(format!("{dimension}.{}", impact_str(impact))).or_insert(0);
    *count += 1;
}

const fn category_priority(category: MismatchCategory) -> u8 {
    match category {
        MismatchCategory::NewBug => 0,
        MismatchCategory::HumanReview => 1,
        MismatchCategory::InsufficientData => 2,
        MismatchCategory::IntentionalChange => 3,
        MismatchCategory::OldBugFixed => 4,
        MismatchCategory::OldBugPreserved => 5,
        MismatchCategory::Compatible => 6,
    }
}

fn note_for_finding(category: MismatchCategory, notes: &[String]) -> String {
    if !notes.is_empty() {
        return notes.join("; ");
    }

    if category == MismatchCategory::HumanReview {
        return "Review top human_review diffs before accepting parity.".to_owned();
    }

    "No note recorded.".to_owned()
}

fn push_count_lines(markdown: &mut String, counts: &BTreeMap<String, usize>) {
    if counts.is_empty() {
        markdown.push_str("- None\n");
        return;
    }

    for (key, count) in counts {
        let _ = writeln!(markdown, "- `{key}`: {count}");
    }
}

const fn impact_str(impact: ImpactLevel) -> &'static str {
    match impact {
        ImpactLevel::Yes => "yes",
        ImpactLevel::No => "no",
        ImpactLevel::Unknown => "unknown",
    }
}
