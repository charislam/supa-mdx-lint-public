use log::debug;
use markdown::mdast::Node;
use supa_mdx_macros::RuleName;

use crate::{
    context::Context,
    errors::{LintError, LintLevel},
    fix::{LintCorrection, LintCorrectionInsert},
    location::{AdjustedRange, DenormalizedLocation},
};

use super::{Rule, RuleName, RuleSettings};

/// Ensure lines inside admonitions are separated by blank lines.
#[derive(Debug, Default, RuleName)]
pub struct Rule006AdmonitionLineSeparation;

impl Rule for Rule006AdmonitionLineSeparation {
    fn default_level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn setup(&mut self, _settings: Option<&mut RuleSettings>) {}

    fn check(&self, ast: &Node, context: &Context, level: LintLevel) -> Option<Vec<LintError>> {
        if let Node::MdxJsxFlowElement(element) = ast {
            if element
                .name
                .as_ref()
                .is_some_and(|name| name == "Admonition")
            {
                if let Some(error) = self.check_admonition(element, context, level) {
                    return Some(vec![error]);
                }
            }
        }
        None
    }
}

impl Rule006AdmonitionLineSeparation {
    fn check_admonition(
        &self,
        element: &markdown::mdast::MdxJsxFlowElement,
        context: &Context,
        level: LintLevel,
    ) -> Option<LintError> {
        if element.children.is_empty() {
            return None;
        }

        let position = element.position.as_ref()?;
        let adjusted_range = AdjustedRange::from_unadjusted_position(position, context);

        let rope = context.rope();
        let range: std::ops::Range<usize> = adjusted_range.clone().into();
        let content = rope.byte_slice(range).to_string();
        debug!("Admonition content: {:?}", content);

        if let Some(fixes) = self.generate_fixes(&content, &adjusted_range, context) {
            let location =
                DenormalizedLocation::from_offset_range(adjusted_range, context);
            return Some(
                LintError::from_raw_location()
                    .rule(self.name())
                    .message("Lines in admonitions must be separated by blank lines")
                    .level(level)
                    .location(location)
                    .fix(fixes)
                    .call(),
            );
        }

        None
    }

    fn generate_fixes(
        &self,
        content: &str,
        adjusted_range: &AdjustedRange,
        context: &Context,
    ) -> Option<Vec<LintCorrection>> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() < 3 {
            return None;
        }

        let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
        let line_ending_len = line_ending.len();
        let mut fixes = Vec::new();
        let mut offset = lines[0].len() + line_ending_len;
        let mut last_was_content = false;

        for line in &lines[1..] {
            if line.trim_start().starts_with("</Admonition") {
                break;
            }
            if line.trim().is_empty() {
                last_was_content = false;
                offset += line.len() + line_ending_len;
                continue;
            }
            if last_was_content {
                let mut start = adjusted_range.start;
                start.increment(offset);
                let location = DenormalizedLocation::from_offset_range(
                    AdjustedRange::new(start, start),
                    context,
                );
                fixes.push(LintCorrection::Insert(LintCorrectionInsert {
                    location,
                    text: line_ending.to_string(),
                }));
                offset += line.len() + line_ending_len;
                last_was_content = true; // already set
                continue;
            }
            last_was_content = true;
            offset += line.len() + line_ending_len;
        }

        if fixes.is_empty() { None } else { Some(fixes) }
    }
}
